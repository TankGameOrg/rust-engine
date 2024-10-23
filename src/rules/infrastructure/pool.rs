use std::{collections::HashMap, error::Error, marker::PhantomData, sync::atomic::{AtomicUsize, Ordering}};

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
}

pub trait AttributeIndex<T: AttributeValue> {
    fn add_container(&mut self, handle: Handle, new_value: &T);
    fn update_container(&mut self, handle: Handle, old_value: &T, new_value: &T);
}

struct AttributeIndexWrapper<T: AttributeValue, IndexType: AttributeIndex<T>> {
    index: IndexType,
    phantom: PhantomData<T>,
}

impl<T: AttributeValue, IndexType: AttributeIndex<T>> AttributeIndexWrapper<T, IndexType> {
    fn new(index: IndexType) -> AttributeIndexWrapper<T, IndexType> {
        AttributeIndexWrapper {
            index,
            phantom: PhantomData,
        }
    }
}

impl<T: AttributeValue, IndexType: AttributeIndex<T> + 'static> Index for AttributeIndexWrapper<T, IndexType> {
    fn add_container(&mut self, handle: Handle, new_value: &dyn AttributeValue) {
        // TODO: Proper type checks?
        self.index.add_container(handle, new_value.downcast_ref::<T>().unwrap());
    }

    fn update_container(&mut self, handle: Handle, old_value: &dyn AttributeValue, new_value: &dyn AttributeValue) {
        // TODO: Proper type checks?
        self.index.update_container(handle, old_value.downcast_ref::<T>().unwrap(), new_value.downcast_ref::<T>().unwrap());
    }
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

    pub fn get_index<T: AttributeValue>(&self, attribute: &Attribute<T>) -> Option<&dyn Index> {
        self.indexes.get(attribute.get_name()).map(|index| index.as_ref())
    }

    pub(super) fn get_index_mut<T: AttributeValue>(&mut self, attribute: &Attribute<T>) -> Option<&mut Box<dyn Index>> {
        self.indexes.get_mut(attribute.get_name())
    }

    #[inline]
    pub fn add_index_typed<T: AttributeValue>(&mut self, attribute: &Attribute<T>, index: impl AttributeIndex<T> + 'static) {
        self.add_index(attribute, AttributeIndexWrapper::new(index));
    }

    #[inline]
    pub fn add_index<T: AttributeValue>(&mut self, attribute: &Attribute<T>, index: impl Index + 'static) {
        self.indexes.insert(attribute.get_name(), Box::new(index));
    }
}

impl std::fmt::Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pool ")?;
        self.containers.fmt(f)
    }
}

pub fn get_index_as<'index, T: AttributeValue, IndexType: Index + 'static>(pool: &'index Pool, attribute: &Attribute<T>) -> Result<&'index IndexType, Box<dyn Error>> {
    match pool.get_index(attribute) {
        Some(index) => {
            match index.downcast_ref::<IndexType>() {
                Some(index) => Ok(index),
                None => Err(Box::new(RuleError::Generic(format!("Expected index for {} to be {} but got type {:?}", attribute.get_name(), stringify!(IndexType), index.type_id())))),
            }
        },
        None => Err(Box::new(RuleError::Generic(format!("Could not find an index for {}", attribute.get_name())))),
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashSet, error::Error};

    use crate::{modify_container, rules::infrastructure::{attribute::DUMMY_ATTRIBUTE, transaction::Transaction}};

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


    struct HasAttributeIndex {
        containers: HashSet<Handle>,
    }

    impl HasAttributeIndex {
        fn new() -> HasAttributeIndex {
            return HasAttributeIndex {
                containers: HashSet::new(),
            }
        }
    }

    impl Index for HasAttributeIndex {
        fn add_container(&mut self, handle: Handle, _new_value: &dyn AttributeValue) {
            self.containers.insert(handle);
        }

        fn update_container(&mut self, _handle: Handle, _old_value: &dyn AttributeValue, _new_value: &dyn AttributeValue) {}
    }

    fn query_has_attribute<'iter, T: AttributeValue>(pool: &'iter Pool, attribute: &Attribute<T>) -> Result<Vec<GatheredResult<'iter>>, Box<dyn Error>> {
        let index: &HasAttributeIndex = get_index_as(pool, attribute)?;

        index.containers.iter()
            .map(|handle| {
                Ok(GatheredResult {
                    handle: *handle,
                    container: pool.get_attribute_container(*handle)?,
                })
            })
            .collect()
    }

    #[test]
    fn index_test() {
        let mut pool = Pool::new();
        pool.add_index(&DUMMY_ATTRIBUTE, HasAttributeIndex::new());

        pool.add_attribute_container();

        let handle = pool.add_attribute_container();

        let mut transaction = Transaction::new();
        modify_container!(&mut transaction, handle, {
            DUMMY_ATTRIBUTE = 2
        });

        transaction.apply(&mut pool).unwrap();

        let matches = query_has_attribute(&pool, &DUMMY_ATTRIBUTE).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].handle, handle);
    }
}