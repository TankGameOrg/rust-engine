use std::{
    collections::HashMap,
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
};

use as_any::{AsAny, Downcast};

use crate::rules::infrastructure::error::RuleError;

use super::{
    attribute::{AnyAttribute, Attribute, AttributeValue},
    entity::Entity,
};

/// A handle can be used to access and modify an Entity in a Universe
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

/// GenericIndex is the internal, boxable, representation of an index
///
/// It allows us to store Indicies with multiple AttributeValue types in the same HashMap
pub(super) trait AnyIndex: AsAny {
    fn add_attribute_hook(
        &mut self,
        handle: Handle,
        new_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>>;
    fn update_attribute_hook(
        &mut self,
        handle: Handle,
        old_value: &dyn AttributeValue,
        new_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>>;
    fn remove_attribute_hook(
        &mut self,
        handle: Handle,
        old_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>>;
}

/// A type that can optimize searches for entities with a specified attribute
///
/// The Index trait provides a set of methods to update the index when an attribute changes
/// but it does not provide an api for querying the index.  It is assumed that users will downcast
/// the index and call an index specific query API.
pub trait Index: AsAny {
    type AttributeValueType;

    /// Start tracking a entity after the attribute this index tracks has been added to it
    ///
    /// `add_attribute` can only be called with handles that are not currently store by this index.
    /// so add_attribute add_attribute is invalid but add_attribute remove_attribute add_attribute is valid.
    ///
    /// If an error is returned, the transaction that triggered the entity add will not be applied
    fn add_attribute(
        &mut self,
        handle: Handle,
        new_value: &Self::AttributeValueType,
    ) -> Result<(), Box<dyn Error>>;

    /// The value of the attribute that this index tracks has been updated
    ///
    /// `update_attribute` can only be called with handles that are tracked by the Index (i.e. `add_attirbute`
    /// has already been called).  Additionally `old_value` must match the `new_value` given to the most recent
    /// `add_attribute` or `update_attribute` call.
    ///
    /// If an error is returned, the transaction that triggered the entity update will not be applied
    fn update_attribute(
        &mut self,
        handle: Handle,
        old_value: &Self::AttributeValueType,
        new_value: &Self::AttributeValueType,
    ) -> Result<(), Box<dyn Error>> {
        self.remove_attribute(handle, old_value)?;
        self.add_attribute(handle, new_value)
    }

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
        handle: Handle,
        old_value: &Self::AttributeValueType,
    ) -> Result<(), Box<dyn Error>>;
}

impl<F: Index> AnyIndex for F {
    fn add_attribute_hook(
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

        self.add_attribute(handle, new_value)
    }

    fn update_attribute_hook(
        &mut self,
        handle: Handle,
        old_value: &dyn AttributeValue,
        new_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>> {
        let old_value = old_value
            .downcast_ref()
            .ok_or(Box::new(RuleError::Generic(format!(
                "Failed to cast old_value to {} from {:?}",
                stringify!(AttributeValueType),
                old_value.type_id()
            ))))?;

        let new_value = new_value
            .downcast_ref()
            .ok_or(Box::new(RuleError::Generic(format!(
                "Failed to cast new_value to {} from {:?}",
                stringify!(AttributeValueType),
                new_value.type_id()
            ))))?;

        self.update_attribute(handle, old_value, new_value)
    }

    fn remove_attribute_hook(
        &mut self,
        handle: Handle,
        old_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>> {
        let old_value = old_value
            .downcast_ref()
            .ok_or(Box::new(RuleError::Generic(format!(
                "Failed to cast old_value to {} from {:?}",
                stringify!(AttributeValueType),
                old_value.type_id()
            ))))?;

        self.remove_attribute(handle, old_value)
    }
}

/// The entity and handle that matched an index or gather filter
pub struct GatheredResult<'entity> {
    pub handle: Handle,
    pub entity: &'entity Entity,
}

/// A collection of entities that can be queried by their attributes
pub struct Universe {
    entities: HashMap<Handle, Entity>,
    indicies: HashMap<&'static dyn AnyAttribute, Box<dyn AnyIndex>>,
    is_valid: bool,
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
            is_valid: true,
        }
    }

    /// Verify that the none of the transactions applied to the universe failed and return an error if one has
    #[inline]
    fn assert_validity(&self) -> Result<(), Box<dyn Error>> {
        if self.is_valid {
            Ok(())
        } else {
            Err(Box::new(RuleError::Generic(String::from(
                "A transaction failed to apply so this universe is no longer in a known good state",
            ))))
        }
    }

    // Set the universe to an invalid state due to a transaction failing to apply
    pub(super) fn set_transaction_failed(&mut self) {
        self.is_valid = false;
    }

    /// Add an Entity with an existing handle
    ///
    /// This method exists to allow the CreateEntityModification to return a handle when it's created even though the entity
    /// itself hasn't been created yet
    #[inline]
    pub(super) fn add_entity(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
        self.assert_validity()?;

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

    /// Get the Entity pointed to by a handle
    ///
    /// If the entity does not exist we return an error
    #[inline]
    pub fn get_entity(&self, handle: Handle) -> Result<&Entity, Box<dyn Error>> {
        self.assert_validity()?;

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
    pub(super) fn get_entity_mut(&mut self, handle: Handle) -> Result<&mut Entity, Box<dyn Error>> {
        self.assert_validity()?;

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
    pub(super) fn remove_entity(&mut self, handle: Handle) -> Result<Entity, Box<dyn Error>> {
        self.assert_validity()?;

        let optional_entity = self.entities.remove(&handle);

        match optional_entity {
            None => Err(Box::new(RuleError::Generic(format!(
                "The handle {:?} does not reference a valid entity",
                handle
            )))),
            Some(entity) => Ok(entity),
        }
    }

    /// Filter all of the entities in the universe and return an iterator to the ones that match
    pub fn gather<'iter>(
        &'iter self,
        predicate: &'iter dyn Fn(&Entity) -> bool,
    ) -> Result<impl Iterator<Item = GatheredResult<'iter>>, Box<dyn Error>> {
        self.assert_validity()?;

        Ok(self
            .entities
            .iter()
            .filter(|(_, entity)| predicate(entity))
            .map(|(handle, entity)| GatheredResult {
                handle: *handle,
                entity,
            }))
    }

    /// Gather the entities assosiated with an iterable of handles
    ///
    /// Return an error if any of the entities doesn't exist
    pub fn gather_handles<'iter>(
        &self,
        iter: impl Iterator<Item = &'iter Handle>,
    ) -> Result<Vec<GatheredResult>, Box<dyn Error>> {
        self.assert_validity()?;

        iter.map(|handle| {
            Ok(GatheredResult {
                handle: *handle,
                entity: self.get_entity(*handle)?,
            })
        })
        .collect()
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
        self.assert_validity()?;

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

    /// Get a mutable refrence to an index to update it to handle a modification to a entity
    #[inline]
    pub(super) fn get_index_mut(
        &mut self,
        attribute: &dyn AnyAttribute,
    ) -> Option<&mut Box<dyn AnyIndex>> {
        if self.is_valid {
            self.indicies.get_mut(attribute)
        } else {
            None
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
        self.assert_validity().unwrap();
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

        let entity = universe.get_entity_mut(handle).unwrap();
        entity.set(&DUMMY_ATTRIBUTE, 2);

        let entity = universe.get_entity(handle).unwrap();
        assert_eq!(*entity.get(&DUMMY_ATTRIBUTE).unwrap(), 2);
    }

    #[test]
    fn can_add_a_entity_with_an_existing_handle() {
        let mut universe = Universe::new();
        let handle = Handle::new();

        universe.add_entity(handle).unwrap();
        universe.get_entity(handle).unwrap();

        let error = universe.add_entity(handle);
        assert!(error.is_err());
    }

    #[test]
    fn can_gather_entities() {
        let mut universe = Universe::new();
        let first_handle = Handle::new();
        universe.add_entity(first_handle).unwrap();
        let first = universe.get_entity_mut(first_handle).unwrap();
        first.set(&DUMMY_ATTRIBUTE, 2);

        let second_handle = Handle::new();
        universe.add_entity(second_handle).unwrap();
        let second = universe.get_entity_mut(second_handle).unwrap();
        second.set(&DUMMY_ATTRIBUTE, 1);

        universe.add_entity(Handle::new()).unwrap();

        // Gather one of the entities
        let one: Vec<GatheredResult> = universe
            .gather(&|entity| {
                *entity
                    .get(&DUMMY_ATTRIBUTE)
                    .or_else(|_| -> Result<&u32, Box<dyn Error>> { Ok(&5) })
                    .unwrap()
                    < 2
            })
            .unwrap()
            .collect();

        assert_eq!(one.len(), 1);
        assert_eq!(one[0].handle, second_handle);
        assert_eq!(*one[0].entity.get(&DUMMY_ATTRIBUTE).unwrap(), 1);

        // Gather both of the ones with attributes
        let two: Vec<Handle> = universe
            .gather(&|entity| entity.has(&DUMMY_ATTRIBUTE))
            .unwrap()
            .map(|result| result.handle)
            .collect();

        println!("{:?} - {:?}, {:?}", two, first_handle, second_handle);
        assert_eq!(two.len(), 2);
        assert!(two.contains(&first_handle));
        assert!(two.contains(&second_handle));
    }

    #[test]
    fn can_gather_entities_from_handles() {
        let mut universe = Universe::new();
        let first_handle = Handle::new();
        universe.add_entity(first_handle).unwrap();
        let first: &mut Entity = universe.get_entity_mut(first_handle).unwrap();
        first.set(&DUMMY_ATTRIBUTE, 2);

        let second_handle = Handle::new();
        universe.add_entity(second_handle).unwrap();
        let second = universe.get_entity_mut(second_handle).unwrap();
        second.set(&DUMMY_ATTRIBUTE, 1);

        universe.add_entity(Handle::new()).unwrap();

        // Gather two of the entities
        let matches = universe
            .gather_handles([first_handle, second_handle].iter())
            .unwrap();

        assert_eq!(matches.len(), 2);

        let handles: Vec<Handle> = matches.iter().map(|result| result.handle).collect();
        assert!(handles.contains(&first_handle));
        assert!(handles.contains(&second_handle));

        let attributes: Vec<u32> = matches
            .iter()
            .map(|result| *result.entity.get(&DUMMY_ATTRIBUTE).unwrap())
            .collect();
        assert!(attributes.contains(&1));
        assert!(attributes.contains(&2));
    }

    struct DummyIndex;

    impl Index for DummyIndex {
        type AttributeValueType = u32;
        fn add_attribute(
            &mut self,
            _handle: Handle,
            _new_value: &Self::AttributeValueType,
        ) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        fn remove_attribute(
            &mut self,
            _handle: Handle,
            _old_value: &Self::AttributeValueType,
        ) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        fn update_attribute(
            &mut self,
            _handle: Handle,
            _old_value: &Self::AttributeValueType,
            _new_value: &Self::AttributeValueType,
        ) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }

    #[test]
    fn failed_transaction_invalidates_universe() {
        let mut universe = Universe::new();

        let handle = Handle::new();
        universe.add_entity(handle).unwrap();
        universe
            .get_entity_mut(handle)
            .unwrap()
            .set(&DUMMY_ATTRIBUTE, 9);

        universe.set_transaction_failed();

        assert!(universe.add_entity(Handle::new()).is_err());
        assert!(universe.get_entity(handle).is_err());
        assert!(universe.get_entity_mut(handle).is_err());
        assert!(universe.remove_entity(handle).is_err());
        assert!(universe.gather(&|_c| true).is_err());
        assert!(universe.gather_handles([handle].iter()).is_err());
        let result: Result<&DummyIndex, Box<dyn Error>> = universe.get_index(&DUMMY_ATTRIBUTE);
        assert!(result.is_err());
    }
}
