use std::{collections::HashMap, error::Error};

use as_any::{AsAny, Downcast};

use crate::rules::infrastructure::error::RuleError;

use super::{
    attribute::{AnyAttribute, Attribute, AttributeValue, IndexedBy},
    entity::Entity,
    index::{Handle, Index},
};

/// GenericIndex is the internal, boxable, representation of an index
///
/// It allows us to store Indicies with multiple AttributeValue types in the same HashMap
pub trait AnyIndex: AsAny {
    fn set_attribute_dynamic(
        &mut self,
        handle: Handle,
        new_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>>;
    fn remove_attribute_dynamic(&mut self, handle: Handle) -> Result<(), Box<dyn Error>>;
}

impl<F: Index> AnyIndex for F {
    fn set_attribute_dynamic(
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

    fn remove_attribute_dynamic(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
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
    pub fn get_attribute<T: AttributeValue>(
        &self,
        handle: Handle,
        key: &dyn Attribute<T>,
    ) -> Result<&T, Box<dyn Error>> {
        self.get_entity(handle)?.get(key)
    }

    /// Set an attribute's value for an entity
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn set_attribute<T: AttributeValue>(
        &mut self,
        handle: Handle,
        key: &'static dyn Attribute<T>,
        value: T,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(index) = self.indicies.get_mut(key.as_any_attribute()) {
            index.set_attribute_dynamic(handle, &value)?;
        }

        self.get_entity_mut(handle)?.set(key, value);
        Ok(())
    }

    /// Remove an attribute from an entity
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn remove_attribute<T: AttributeValue>(
        &mut self,
        handle: Handle,
        key: &dyn Attribute<T>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(index) = self.indicies.get_mut(key.as_any_attribute()) {
            index.remove_attribute_dynamic(handle)?;
        }

        self.get_entity_mut(handle)?.remove(key.as_any_attribute());
        Ok(())
    }

    /// Check an entity has an attribute
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn has_attribute(
        &self,
        handle: Handle,
        key: &dyn AnyAttribute,
    ) -> Result<bool, Box<dyn Error>> {
        Ok(self.get_entity(handle)?.has(key))
    }

    /// Iterate the attributes on an entity
    #[inline]
    pub fn iter_attributes(
        &self,
        handle: Handle,
    ) -> Result<impl Iterator<Item = (&dyn AnyAttribute, &dyn AttributeValue)>, Box<dyn Error>>
    {
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
                        index.remove_attribute_dynamic(handle)?;
                    }
                }

                Ok(())
            }
        }
    }

    /// Filter all of the entities in the universe and return an iterator to the ones that match
    pub fn gather<'iter>(
        &'iter self,
        predicate: &'iter dyn Fn(Handle) -> bool,
    ) -> impl Iterator<Item = Handle> + 'iter {
        self.entities
            .keys()
            .filter(|handle| predicate(**handle))
            .copied()
    }

    /// Get an index which can be used to find one or more entities based on a specific attribute
    pub fn get_index<ValueType, IndexType>(
        &self,
        attribute: &'static dyn IndexedBy<ValueType, IndexType>,
    ) -> &IndexType
    where
        ValueType: AttributeValue,
        IndexType: Index<AttributeValueType = ValueType> + 'static,
    {
        match self.indicies.get(attribute.as_any_attribute()) {
            Some(index) => match index.as_ref().downcast_ref::<IndexType>() {
                Some(index) => index,
                None => panic!(
                    "Expected index for {} to be {} but got type {:?}",
                    attribute.get_name(),
                    stringify!(IndexType),
                    index.as_ref().type_id()
                ),
            },
            None => panic!(
                "Could not find an index for {}",
                attribute.get_name()
            ),
        }
    }

    /// Add an index to optimize queries for entities with a specific attribute
    ///
    /// All indicies must be added before any entities are and each attribute can only have one index
    #[inline]
    pub fn add_index<
        ValueType: AttributeValue,
        IndexType: Index<AttributeValueType = ValueType>,
    >(
        &mut self,
        attribute: &'static dyn IndexedBy<ValueType, IndexType>,
        index: IndexType,
    ) {
        assert!(
            self.entities.is_empty(),
            "Index for {:?} was added after entities had been added",
            attribute
        );
        assert!(
            !self.indicies.contains_key(attribute.as_any_attribute()),
            "An index has already been registered for {:?}",
            attribute
        );
        self.indicies
            .insert(attribute.as_any_attribute(), Box::new(index));
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

    use crate::{attribute, rules::infrastructure::ecs::attribute::DummyAttribute};

    use super::*;

    #[test]
    fn can_modify_and_retrieve_entities() {
        let mut universe = Universe::new();
        let handle = Handle::new();
        universe.add_entity(handle).unwrap();
        universe.set_attribute(handle, &DummyAttribute, 2).unwrap();

        assert_eq!(*universe.get_attribute(handle, &DummyAttribute).unwrap(), 2);
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
        universe
            .set_attribute(first_handle, &DummyAttribute, 2)
            .unwrap();

        let second_handle = Handle::new();
        universe.add_entity(second_handle).unwrap();
        universe
            .set_attribute(second_handle, &DummyAttribute, 1)
            .unwrap();

        universe.add_entity(Handle::new()).unwrap();

        // Gather one of the entities
        let one: Vec<Handle> = universe
            .gather(&|handle| {
                *universe
                    .get_attribute(handle, &DummyAttribute)
                    .unwrap_or(&5)
                    < 2
            })
            .collect();

        assert_eq!(one.len(), 1);
        assert_eq!(one[0], second_handle);

        // Gather both of the ones with attributes
        let two: Vec<Handle> = universe
            .gather(&|handle| {
                universe
                    .has_attribute(handle, &DummyAttribute)
                    .unwrap_or(false)
            })
            .collect();

        println!("{:?} - {:?}, {:?}", two, first_handle, second_handle);
        assert_eq!(two.len(), 2);
        assert!(two.contains(&first_handle));
        assert!(two.contains(&second_handle));
    }

    struct DummyIndex;

    impl Index for DummyIndex {
        type AttributeValueType = u32;
        fn remove_attribute(&mut self, _handle: Handle) -> Result<(), Box<dyn Error>> {
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

    struct TestIndex {
        handle: Option<Handle>,
    }

    impl TestIndex {
        fn new() -> TestIndex {
            TestIndex { handle: None }
        }

        fn get(&self) -> Result<Handle, Box<dyn Error>> {
            match self.handle {
                None => Err(Box::new(RuleError::Generic(String::from(
                    "No handle stored yet",
                )))),
                Some(handle) => Ok(handle),
            }
        }
    }

    impl Index for TestIndex {
        type AttributeValueType = u32;

        fn set_attribute(
            &mut self,
            handle: Handle,
            _new_value: &u32,
        ) -> Result<(), Box<dyn Error>> {
            self.handle = Some(handle);
            Ok(())
        }

        fn remove_attribute(&mut self, _handle: Handle) -> Result<(), Box<dyn Error>> {
            self.handle = None;
            Ok(())
        }
    }

    attribute!(DummyAttribute2: u32, indexed by TestIndex);

    #[test]
    fn index_test() {
        let mut universe = Universe::new();
        universe.add_index(&DummyAttribute2, TestIndex::new());

        let handle = Handle::new();
        universe.add_entity(handle).unwrap();
        universe.set_attribute(handle, &DummyAttribute, 2).unwrap();
        universe.set_attribute(handle, &DummyAttribute2, 6).unwrap();

        universe.add_entity(Handle::new()).unwrap();

        let result = universe.get_index(&DummyAttribute2).get().unwrap();
        assert_eq!(result, handle);

        universe.remove_entity(handle).unwrap();
    }

    struct FailingIndex {}

    static mut FAILING_INDEX_FAILS: bool = false;

    impl FailingIndex {
        fn return_result(&self) -> Result<(), Box<dyn Error>> {
            if unsafe { FAILING_INDEX_FAILS } {
                Err(Box::new(RuleError::Generic(String::from("Tripped error"))))
            } else {
                Ok(())
            }
        }
    }

    impl Index for FailingIndex {
        type AttributeValueType = u32;

        fn remove_attribute(&mut self, _handle: Handle) -> Result<(), Box<dyn Error>> {
            self.return_result()
        }

        fn set_attribute(
            &mut self,
            _handle: Handle,
            _new_value: &Self::AttributeValueType,
        ) -> Result<(), Box<dyn Error>> {
            self.return_result()
        }
    }

    attribute!(FailingAttribute: u32, indexed by FailingIndex);

    #[test]
    fn failing_index() {
        let mut universe = Universe::new();
        universe.add_index(&FailingAttribute, FailingIndex {});

        let handle = Handle::new();
        universe.add_entity(handle).unwrap();

        let handle2 = Handle::new();
        universe.add_entity(handle2).unwrap();
        universe
            .set_attribute(handle2, &FailingAttribute, 1)
            .unwrap();

        universe.set_attribute(handle, &FailingAttribute, 2).unwrap();

        unsafe {
            FAILING_INDEX_FAILS = true;
        }

        assert!(universe.remove_entity(handle).is_err());
    }
}
