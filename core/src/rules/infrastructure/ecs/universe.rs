use std::{
    any::Any, collections::HashMap, error::Error, sync::atomic::{AtomicUsize, Ordering}
};

use as_any::{AsAny, Downcast};

use crate::rules::infrastructure::error::RuleError;

use super::{attribute::{AnyAttribute, Attribute, AttributeValue}, entity::Entity};


/// A handle can be used to access and modify an Entity in a Universe
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Handle(usize);

static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0);

impl Handle {
    // TODO: PRIVATE
    #[inline]
    pub fn new() -> Handle {
        Handle(NEXT_HANDLE.fetch_add(1, Ordering::Relaxed))
    }
}

impl AttributeValue for Handle {}

/// GenericIndex is the internal, boxable, representation of an index
///
/// It allows us to store Indicies with multiple AttributeValue types in the same HashMap
pub trait AnyIndex: AsAny {
    fn set_attribute_hook(
        &mut self,
        handle: Handle,
        new_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>>;
    fn remove_attribute_hook(
        &mut self,
        handle: Handle
    ) -> Result<(), Box<dyn Error>>;
}

/// A type that can optimize searches for entities with a specified attribute
///
/// The Index trait provides a set of methods to update the index when an attribute changes
/// but it does not provide an api for querying the index.  It is assumed that users will downcast
/// the index and call an index specific query API.
pub trait Index: AsAny {
    type AttributeValueType;

    /// The value of the attribute that this index tracks has been updated
    ///
    /// `update_attribute` can only be called with handles that are tracked by the Index (i.e. `add_attirbute`
    /// has already been called).  Additionally `old_value` must match the `new_value` given to the most recent
    /// `add_attribute` or `update_attribute` call.
    ///
    /// If an error is returned, the transaction that triggered the entity update will not be applied
    fn set_attribute(
        &mut self,
        handle: Handle,
        new_value: &Self::AttributeValueType,
    ) -> Result<(), Box<dyn Error>>;

    /// Stop tracking a entity after the attribute this index tracks was removed
    ///
    /// `remove_attribute` can only be called with handles that are tracked by the Index (i.e. `add_attirbute`
    /// has already been called).  Additionally `old_value` must match the `new_value` given to the most recent
    /// `add_attribute` or `update_attribute` call.  After `remove_attribute` is called the handle is no longer
    /// tracked by the index.
    ///
    /// If an error is returned, the transaction that triggered the entity remove will not still be applied
    fn remove_attribute(
        &mut self,
        handle: Handle
    ) -> Result<(), Box<dyn Error>>;
}

impl<F: Index> AnyIndex for F {
    fn set_attribute_hook(
        &mut self,
        handle: Handle,
        new_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>> {
        let new_value = new_value
            .downcast_ref()
            .ok_or(Box::new(RuleError::Generic(format!(
                "Failed to cast new_value to {} from {:?}",
                stringify!(AttributeValueType),
                new_value.type_id()
            ))))?;

        self.set_attribute(handle, new_value)
    }

    fn remove_attribute_hook(
        &mut self,
        handle: Handle
    ) -> Result<(), Box<dyn Error>> {
        self.remove_attribute(handle)
    }
}

/// A collection of entities that can be queried by their attributes
pub struct Universe {
    entities: HashMap<Handle, Entity>,
    indicies: HashMap<&'static dyn AnyAttribute, Box<dyn AnyIndex>>,
}

impl Default for Universe {
    fn default() -> Self {
        Self::new()
    }
}

impl Universe {
    #[inline]
    pub fn new() -> Universe {
        Universe {
            entities: HashMap::new(),
            indicies: HashMap::new(),
        }
    }

    /// Add an Entity with an existing handle
    ///
    /// This method exists to allow the CreateEntityModification to return a handle when it's created even though the entity
    /// itself hasn't been created yet
    #[inline]
    pub fn add_entity(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
        if self.entities.contains_key(&handle) {
            let current = self.entities.get(&handle).unwrap();
            return Err(Box::new(RuleError::Generic(format!(
                "The handle {:?} already exists in this universe (current = {:?})",
                handle, current
            ))));
        }

        self.entities.insert(handle, Entity::new());
        Ok(())
    }

    /// Get an attribute's value from an entity
    /// 
    /// If the entity doesn't exist or it doesn't exist we return an error
    #[inline]
    pub fn get_attribute<T: AttributeValue>(&self, handle: Handle, key: &Attribute<T>) -> Result<&T, Box<dyn Error>> {
        self.get_entity(handle)?.get(key)
    }

    /// Set an attribute's value for an entity
    /// 
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn set_attribute<T: AttributeValue>(&mut self, handle: Handle, key: &'static Attribute<T>, value: T) -> Result<(), Box<dyn Error>> {
        if let Some(index) = self.indicies.get_mut(key as &dyn AnyAttribute) {
            index.set_attribute_hook(handle, &value)?;
        }

        self.get_entity_mut(handle)?.set(key, value);
        Ok(())
    }

    /// Remove an attribute from an entity
    /// 
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn remove_attribute<T: AttributeValue>(&mut self, handle: Handle, key: &Attribute<T>) -> Result<(), Box<dyn Error>> {
        if let Some(index) = self.indicies.get_mut(key as &dyn AnyAttribute) {
            index.remove_attribute_hook(handle)?;
        }

        self.get_entity_mut(handle)?.remove(key);
        Ok(())
    }

    /// Check an entity has an attribute
    /// 
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn has_attribute(&self, handle: Handle, key: &dyn AnyAttribute) -> Result<bool, Box<dyn Error>> {
        Ok(self.get_entity(handle)?.has(key))
    }

    /// Iterate the attributes on an entity
    #[inline]
    pub fn iter_attributes(&self, handle: Handle) -> Result<impl Iterator<Item = (&dyn AnyAttribute, &dyn AttributeValue)>, Box<dyn Error>> {
        Ok(self.get_entity(handle)?.iter())
    }

    /// Get the Entity pointed to by a handle
    ///
    /// If the entity does not exist we return an error
    #[inline]
    fn get_entity(&self, handle: Handle) -> Result<&Entity, Box<dyn Error>> {
        self.entities
            .get(&handle)
            .ok_or(Box::new(RuleError::Generic(format!(
                "Entity for {:?} does not exist",
                handle
            ))))
    }

    /// Get a mutable reference to the Entity pointed to by a haandle
    ///
    /// If the entity does not exist we return an error
    #[inline]
    fn get_entity_mut(&mut self, handle: Handle) -> Result<&mut Entity, Box<dyn Error>> {
        self.entities
            .get_mut(&handle)
            .ok_or(Box::new(RuleError::Generic(format!(
                "Entity for {:?} does not exist",
                handle
            ))))
    }

    /// Remove a entity from a universe
    ///
    /// If the handle does not exist return an error
    pub fn remove_entity(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
        let optional_entity = self.entities.remove(&handle);

        match optional_entity {
            None => Err(Box::new(RuleError::Generic(format!(
                "The handle {:?} does not reference a valid entity",
                handle
            )))),
            Some(entity) => {
                for (attribute, _) in &entity {
                    if let Some(index) = self.indicies.get_mut(attribute) {
                        index.remove_attribute_hook(handle)?;
                    }
                }

                Ok(())
            },
        }
    }

    /// Filter all of the entities in the universe and return an iterator to the ones that match
    pub fn gather<'iter>(
        &'iter self,
        predicate: &'iter dyn Fn(Handle) -> bool,
    ) -> impl Iterator<Item = Handle> + 'iter {
        self
            .entities
            .keys()
            .filter(|handle| predicate(**handle))
            .map(|handle| *handle)
    }

    /// Get an index which can be used to find one or more entities based on a specific attribute
    pub fn get_index<T, IndexType>(
        &self,
        attribute: &Attribute<T>,
    ) -> Result<&IndexType, Box<dyn Error>>
    where
        T: AttributeValue,
        IndexType: Index<AttributeValueType = T> + 'static,
    {
        match self.indicies.get(attribute as &dyn AnyAttribute) {
            Some(index) => match index.as_ref().downcast_ref::<IndexType>() {
                Some(index) => Ok(index),
                None => Err(Box::new(RuleError::Generic(format!(
                    "Expected index for {} to be {} but got type {:?}",
                    attribute.get_name(),
                    stringify!(IndexType),
                    index.type_id()
                )))),
            },
            None => Err(Box::new(RuleError::Generic(format!(
                "Could not find an index for {}",
                attribute.get_name()
            )))),
        }
    }

    /// Add an index to optimize queries for entities with a specific attribute
    ///
    /// All indicies must be added before any entities are and each attribute can only have one index
    #[inline]
    pub fn add_index<T: AttributeValue>(
        &mut self,
        attribute: &'static Attribute<T>,
        index: impl Index<AttributeValueType = T> + 'static,
    ) {
        assert!(
            self.entities.is_empty(),
            "Index for {:?} was added after entities had been added",
            attribute
        );
        assert!(
            !self.indicies.contains_key(attribute as &dyn AnyAttribute),
            "An index has already been registered for {:?}",
            attribute
        );
        self.indicies.insert(attribute, Box::new(index));
    }
}

impl std::fmt::Debug for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Universe ")?;
        self.entities.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::rules::infrastructure::ecs::attribute::DUMMY_ATTRIBUTE;

    use super::*;

    #[test]
    fn can_modify_and_retrieve_entities() {
        let mut universe = Universe::new();
        let handle = Handle::new();
        universe.add_entity(handle).unwrap();
        universe.set_attribute(handle, &DUMMY_ATTRIBUTE, 2).unwrap();

        assert_eq!(*universe.get_attribute(handle, &DUMMY_ATTRIBUTE).unwrap(), 2);
    }

    #[test]
    fn can_add_a_entity_with_an_existing_handle() {
        let mut universe = Universe::new();
        let handle = Handle::new();

        universe.add_entity(handle).unwrap();

        let error = universe.add_entity(handle);
        assert!(error.is_err());
    }

    #[test]
    fn can_gather_entities() {
        let mut universe = Universe::new();
        let first_handle = Handle::new();
        universe.add_entity(first_handle).unwrap();
        universe.set_attribute(first_handle, &DUMMY_ATTRIBUTE, 2).unwrap();

        let second_handle = Handle::new();
        universe.add_entity(second_handle).unwrap();
        universe.set_attribute(second_handle, &DUMMY_ATTRIBUTE, 1).unwrap();

        universe.add_entity(Handle::new()).unwrap();

        // Gather one of the entities
        let one: Vec<Handle> = universe
            .gather(&|handle| *universe.get_attribute(handle, &DUMMY_ATTRIBUTE).unwrap_or(&5) < 2)
            .collect();

        assert_eq!(one.len(), 1);
        assert_eq!(one[0], second_handle);

        // Gather both of the ones with attributes
        let two: Vec<Handle> = universe
            .gather(&|handle| universe.has_attribute(handle, &DUMMY_ATTRIBUTE).unwrap_or(false))
            .collect();

        println!("{:?} - {:?}, {:?}", two, first_handle, second_handle);
        assert_eq!(two.len(), 2);
        assert!(two.contains(&first_handle));
        assert!(two.contains(&second_handle));
    }

    struct DummyIndex;

    impl Index for DummyIndex {
        type AttributeValueType = u32;
        fn remove_attribute(
            &mut self,
            _handle: Handle
        ) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        fn set_attribute(
            &mut self,
            _handle: Handle,
            _new_value: &Self::AttributeValueType,
        ) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }
}
