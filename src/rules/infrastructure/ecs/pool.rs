use std::{collections::HashMap, error::Error, sync::atomic::{AtomicUsize, Ordering}};

use as_any::{AsAny, Downcast};

use crate::rules::infrastructure::error::RuleError;

use super::{attribute::{Attribute, AttributeValue}, container::AttributeContainer};


/// A handle can be used to access and modify an AttributeContainer in a Pool
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Handle(usize);

static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0);

impl Handle {
    #[inline]
    pub(super) fn new() -> Handle {
        Handle(NEXT_HANDLE.fetch_add(1, Ordering::Relaxed))
    }
}

impl AttributeValue for Handle {}

pub trait Index: AsAny {
    fn add_container(&mut self, handle: Handle, new_value: &dyn AttributeValue);
    fn update_container(&mut self, handle: Handle, old_value: &dyn AttributeValue, new_value: &dyn AttributeValue);
    fn remove_container(&mut self, handle: Handle, old_value: &dyn AttributeValue);
}

pub struct GatheredResult<'container> {
    pub handle: Handle,
    pub container: &'container AttributeContainer,
}

/// A collection of attribute containers that can be queried by their attributes
pub struct Pool {
    containers: HashMap<Handle, AttributeContainer>,
    indexes: HashMap<&'static str, Box<dyn Index>>,
}

impl Pool {
    #[inline]
    pub fn new() -> Pool {
        Pool {
            containers: HashMap::new(),
            indexes: HashMap::new(),
        }
    }

    /// Add an attribute container with an existing handle
    ///
    /// This method exists to allow the AddContainerModification to return a handle when it's created even though the container
    /// itself hasn't been created yet
    #[inline]
    pub(super) fn add_attribute_container_with_handle(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
        if self.containers.contains_key(&handle) {
            let current = self.containers.get(&handle).unwrap();
            return Err(Box::new(RuleError::Generic(format!("The handle {:?} already exists in this pool (current = {:?})", handle, current))))
        }

        self.containers.insert(handle, AttributeContainer::new());
        Ok(())
    }

    /// Add an attribute container and return a handle that can be used to access it
    #[inline]
    pub fn add_attribute_container(&mut self) -> Handle {
        let handle = Handle::new();
        self.containers.insert(handle, AttributeContainer::new());
        handle
    }

    /// Get the attribute container pointed to by a handle
    ///
    /// If the container does not exist we return an error
    #[inline]
    pub fn get_attribute_container(&self, handle: Handle) -> Result<&AttributeContainer, Box<dyn Error>> {
        self.containers.get(&handle)
            .ok_or(Box::new(RuleError::Generic(format!("Attribute container for {:?} does not exist", handle))))
    }

    /// Get a mutable reference to the attribute container pointed to by a haandle
    ///
    /// If the container does not exist we return an error
    #[inline]
    pub(super) fn get_attribute_container_mut(&mut self, handle: Handle) -> Result<&mut AttributeContainer, Box<dyn Error>> {
        self.containers.get_mut(&handle)
            .ok_or(Box::new(RuleError::Generic(format!("Attribute container for {:?} does not exist", handle))))
    }

    /// Filter all of the containers in the pool and return an iterator to the ones that match
    pub fn gather<'iter>(&'iter self, predicate: &'iter dyn Fn(&AttributeContainer) -> bool) -> impl Iterator<Item = GatheredResult<'iter>> {
        self.containers.iter()
            .filter(|(_, container)| predicate(*container))
            .map(|(handle, container)| {
                GatheredResult {
                    handle: *handle,
                    container,
                }
            })
    }

    /// Gather the containers assosiated with an iterable of handles
    /// 
    /// Return an error if any of the containers doesn't exist
    pub fn gather_handles<'iter>(&self, iter: impl Iterator<Item=&'iter Handle>) -> Result<Vec<GatheredResult>, Box<dyn Error>> {
        iter.map(|handle| {
            Ok(GatheredResult {
                handle: *handle,
                container: self.get_attribute_container(*handle)?,
            })
        })
        .collect()
    }

    /// Get an index which can be used to find one or more containers based on a specific attribute
    pub fn get_index<IndexType: Index + 'static>(&self, attribute: &Attribute<impl AttributeValue>) -> Result<&IndexType, Box<dyn Error>> {
        match self.indexes.get(attribute.get_name()) {
            Some(index) => {
                match index.as_ref().downcast_ref::<IndexType>() {
                    Some(index) => Ok(index),
                    None => Err(Box::new(RuleError::Generic(format!("Expected index for {} to be {} but got type {:?}", attribute.get_name(), stringify!(IndexType), index.type_id())))),
                }
            },
            None => Err(Box::new(RuleError::Generic(format!("Could not find an index for {}", attribute.get_name())))),
        }
    }

    /// Get a mutable refrence to an index to update it to handle a modification to a container
    #[inline]
    pub(super) fn get_index_mut(&mut self, attribute: &Attribute<impl AttributeValue>) -> Option<&mut Box<dyn Index>> {
        self.indexes.get_mut(attribute.get_name())
    }

    /// Add an index to optimize queries for containers with a specific attribute
    /// 
    /// All indexes must be added before any containers are and each attribute can only have one index
    #[inline]
    pub fn add_index(&mut self, attribute: &Attribute<impl AttributeValue>, index: impl Index + 'static) {
        assert!(self.containers.len() == 0, "Index for {:?} was added after containers had been added", attribute);
        assert!(!self.indexes.contains_key(attribute.get_name()), "An index has already been registered for {:?}", attribute);
        self.indexes.insert(attribute.get_name(), Box::new(index));
    }
}

impl std::fmt::Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pool ")?;
        self.containers.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::rules::infrastructure::ecs::attribute::DUMMY_ATTRIBUTE;

    use super::*;

    #[test]
    fn can_modify_and_retrieve_containers() {
        let mut pool = Pool::new();
        let handle = pool.add_attribute_container();

        let container = pool.get_attribute_container_mut(handle).unwrap();
        container.set(&DUMMY_ATTRIBUTE, 2);

        let container = pool.get_attribute_container(handle).unwrap();
        assert_eq!(*container.get(&DUMMY_ATTRIBUTE).unwrap(), 2);
    }

    #[test]
    fn can_add_a_container_with_an_existing_handle() {
        let mut pool = Pool::new();
        let handle = Handle::new();

        pool.add_attribute_container_with_handle(handle).unwrap();
        pool.get_attribute_container(handle).unwrap();

        let error = pool.add_attribute_container_with_handle(handle);
        assert!(error.is_err());
    }

    #[test]
    fn can_gather_containers() {
        let mut pool = Pool::new();
        let first_handle = pool.add_attribute_container();
        let first = pool.get_attribute_container_mut(first_handle).unwrap();
        first.set(&DUMMY_ATTRIBUTE, 2);

        let second_handle = pool.add_attribute_container();
        let second = pool.get_attribute_container_mut(second_handle).unwrap();
        second.set(&DUMMY_ATTRIBUTE, 1);

        pool.add_attribute_container();

        // Gather one of the containers
        let one: Vec<GatheredResult> = pool.gather(&|container| {
            *container.get(&DUMMY_ATTRIBUTE)
                .or_else(|_| -> Result<&u32, Box<dyn Error>> { Ok(&5) })
                .unwrap() < 2
        }).collect();

        assert_eq!(one.len(), 1);
        assert_eq!(one[0].handle, second_handle);
        assert_eq!(*one[0].container.get(&DUMMY_ATTRIBUTE).unwrap(), 1);

        // Gather both of the ones with attributes
        let two: Vec<Handle> = pool.gather(&|container| container.has(&DUMMY_ATTRIBUTE))
            .map(|result| result.handle)
            .collect();

        println!("{:?} - {:?}, {:?}", two, first_handle, second_handle);
        assert_eq!(two.len(), 2);
        assert!(two.contains(&first_handle));
        assert!(two.contains(&second_handle));
    }

    #[test]
    fn can_gather_containers_from_handles() {
        let mut pool = Pool::new();
        let first_handle = pool.add_attribute_container();
        let first: &mut AttributeContainer = pool.get_attribute_container_mut(first_handle).unwrap();
        first.set(&DUMMY_ATTRIBUTE, 2);

        let second_handle = pool.add_attribute_container();
        let second = pool.get_attribute_container_mut(second_handle).unwrap();
        second.set(&DUMMY_ATTRIBUTE, 1);

        pool.add_attribute_container();

        // Gather two of the containers
        let matches = pool.gather_handles(vec![first_handle, second_handle].iter()).unwrap();

        assert_eq!(matches.len(), 2);

        let handles: Vec<Handle> = matches.iter().map(|result| result.handle).collect();
        assert!(handles.contains(&first_handle));
        assert!(handles.contains(&second_handle));

        let attributes: Vec<u32> = matches.iter().map(|result| *result.container.get(&DUMMY_ATTRIBUTE).unwrap()).collect();
        assert!(attributes.contains(&1));
        assert!(attributes.contains(&2));
    }
}