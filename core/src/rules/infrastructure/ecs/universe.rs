use std::{collections::HashMap, error::Error};

use as_any::Downcast;

use crate::basic_error;

use super::{
    attribute::{AnyAttribute, Attribute, AttributeValue, IndexedBy},
    entity::Entity,
    index::{AnyIndex, Handle, Index},
};

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

    /// Add an entity and create a new handle
    pub fn add_entity(&mut self) -> Handle {
        let handle = Handle::new();
        self.entities.insert(handle, Entity::new());
        handle
    }

    /// Add an Entity with an existing handle
    ///
    /// Most users should call add_entity() and use the handle it gives you
    pub(in crate::rules::infrastructure) fn add_entity_with_handle(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
        if self.entities.contains_key(&handle) {
            let current = self.entities.get(&handle).unwrap();
            return Err(basic_error!(
                "The handle {:?} already exists in this universe (current = {:?})",
                handle, current
            ));
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
    pub fn set_attribute<T: AttributeValue + Clone>(
        &mut self,
        handle: Handle,
        key: &'static dyn Attribute<T>,
        value: T,
    ) -> Result<(), Box<dyn Error>> {
        let old_value = self.get_entity(handle)?.get(key).ok().cloned();

        if let Some(index) = self.get_index_mut(key) {
            if let Some(old_value) = old_value {
                index.update_attribute_dynamic(handle, &old_value, &value)?;
            }
            else {
                index.add_attribute_dynamic(handle, &value)?;
            }
        }

        self.get_entity_mut(handle)?.set(key, value);
        Ok(())
    }

    /// Remove an attribute from an entity
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn remove_attribute<T: AttributeValue + Clone>(
        &mut self,
        handle: Handle,
        key: &'static dyn Attribute<T>,
    ) -> Result<(), Box<dyn Error>> {
        let old_value = self.get_entity(handle)?.get(key)?.clone();

        if let Some(index) = self.indicies.get_mut(key.as_any_attribute()) {
            index.remove_attribute_dynamic(handle, &old_value)?;
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
            .ok_or(basic_error!(
                "Entity for {:?} does not exist",
                handle
            ))
    }

    /// Get a mutable reference to the Entity pointed to by a haandle
    ///
    /// If the entity does not exist we return an error
    #[inline]
    fn get_entity_mut(&mut self, handle: Handle) -> Result<&mut Entity, Box<dyn Error>> {
        self.entities
            .get_mut(&handle)
            .ok_or(basic_error!(
                "Entity for {:?} does not exist",
                handle
            ))
    }

    /// Remove a entity from a universe
    ///
    /// If the handle does not exist return an error
    pub fn remove_entity(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
        let optional_entity = self.entities.remove(&handle);

        match optional_entity {
            None => Err(basic_error!(
                "The handle {:?} does not reference a valid entity",
                handle
            )),
            Some(entity) => {
                for (attribute, old_value) in &entity {
                    if let Some(index) = self.indicies.get_mut(attribute) {
                        index.remove_attribute_dynamic(handle, old_value)?;
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

    /// Get a mutable reference to this attribute's index if this attribute has one
    /// 
    /// If the attribute has an index but we don't have an instance of it yet it will be created automatically
    fn get_index_mut<T>(&mut self, attribute: &'static dyn Attribute<T>) -> Option<&mut dyn AnyIndex> where T: AttributeValue {
        if !self.indicies.contains_key(attribute.as_any_attribute()) {
            if let Some(new_index) = attribute.create_default_index() {
                self.indicies.insert(attribute.as_any_attribute(), new_index);   
            }
        }

        self.indicies.get_mut(attribute.as_any_attribute()).map(|index| index.as_mut())
    }
}

impl std::fmt::Debug for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Universe ")?;
        self.entities.fmt(f)
    }
}


/// Create an entity with the specified attributes
///
/// ```
/// # use tank_game_core::attribute;
/// # use std::error::Error;
/// # use tank_game_core::rules::infrastructure::ecs::Universe;
/// # use tank_game_core::create_entity;
/// # attribute!(DummyAttribute: u32);
/// # attribute!(DummyAttribute2: u32);
/// #
/// let mut universe = Universe::new();
/// let new_handle = create_entity!(&mut universe, {
///     DummyAttribute = 3,
///     DummyAttribute2 = 4
/// })?;
/// #
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! create_entity {
    ($universe:expr, $($token:tt)*) => {
        {
            use $crate::modify_entity;

            let universe: &mut $crate::rules::infrastructure::ecs::Universe = $universe;

            let handle = universe.add_entity();

            modify_entity!(universe, handle, $($token)*)
                .map(|()| handle)
        }
    };
}

/// Set the specified attributes on an existing entity
///
/// ```
/// # use tank_game_core::attribute;
/// # use std::error::Error;
/// # use tank_game_core::rules::infrastructure::ecs::{Attribute, Universe};
/// # use tank_game_core::{create_entity, modify_entity};
/// # attribute!(DummyAttribute: u32);
/// # attribute!(DummyAttribute2: u32);
/// #
/// let mut universe = Universe::new();
/// # let dummy_handle = create_entity!(&mut universe, { DummyAttribute = 3 })?;
/// modify_entity!(&mut universe, dummy_handle, {
///     DummyAttribute = 3,
///     DummyAttribute2 = 4
/// })?;
/// #
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! modify_entity {
    ($universe:expr, $handle:ident, { $($attribute:ident = $value:expr),+ }) => {
        {
            let universe: &mut $crate::rules::infrastructure::ecs::Universe = $universe;
            let handle: $crate::rules::infrastructure::ecs::Handle = $handle;
            let mut result = Ok(());

            $(
                if result.is_ok() {
                    result = universe.set_attribute(handle, &$attribute, $value);
                }
            )+

            result
        }
    };
}


#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::{attribute, rules::infrastructure::ecs::attribute::DummyAttribute};

    use super::*;

    #[test]
    fn can_modify_and_retrieve_entities() {
        let mut universe = Universe::new();
        let handle = universe.add_entity();
        universe.set_attribute(handle, &DummyAttribute, 2).unwrap();

        assert_eq!(*universe.get_attribute(handle, &DummyAttribute).unwrap(), 2);
    }

    #[test]
    fn can_add_a_entity_with_an_existing_handle() {
        let mut universe = Universe::new();
        let handle = universe.add_entity();

        let error = universe.add_entity_with_handle(handle);
        assert!(error.is_err());
    }

    #[test]
    fn can_gather_entities() {
        let mut universe = Universe::new();
        let first_handle = universe.add_entity();
        universe
            .set_attribute(first_handle, &DummyAttribute, 2)
            .unwrap();

        let second_handle = universe.add_entity();
        universe
            .set_attribute(second_handle, &DummyAttribute, 1)
            .unwrap();

        let _ = universe.add_entity();

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

    #[derive(Default)]
    struct DummyIndex;

    impl Index for DummyIndex {
        type AttributeValueType = u32;
        fn remove_attribute(&mut self, _handle: Handle, _old_value: &u32) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        fn add_attribute(
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
        fn get(&self) -> Result<Handle, Box<dyn Error>> {
            match self.handle {
                None => Err(basic_error!(
                    "No handle stored yet",
                )),
                Some(handle) => Ok(handle),
            }
        }
    }

    impl Default for TestIndex {
        fn default() -> Self {
            TestIndex { handle: None }
        }
    }

    impl Index for TestIndex {
        type AttributeValueType = u32;

        fn add_attribute(
            &mut self,
            handle: Handle,
            _new_value: &u32,
        ) -> Result<(), Box<dyn Error>> {
            self.handle = Some(handle);
            Ok(())
        }

        fn remove_attribute(&mut self, _handle: Handle, _old_value: &u32) -> Result<(), Box<dyn Error>> {
            self.handle = None;
            Ok(())
        }
    }

    attribute!(DummyAttribute2: u32, indexed by TestIndex);

    #[test]
    fn index_test() {
        let mut universe = Universe::new();

        let handle = universe.add_entity();
        universe.set_attribute(handle, &DummyAttribute, 2).unwrap();
        universe.set_attribute(handle, &DummyAttribute2, 6).unwrap();

        let _ = universe.add_entity();

        let result = universe.get_index(&DummyAttribute2).get().unwrap();
        assert_eq!(result, handle);

        universe.remove_entity(handle).unwrap();
    }

    #[derive(Default)]
    struct FailingIndex;

    static mut FAILING_INDEX_FAILS: bool = false;

    impl FailingIndex {
        fn return_result(&self) -> Result<(), Box<dyn Error>> {
            if unsafe { FAILING_INDEX_FAILS } {
                Err(basic_error!("Tripped error"))
            } else {
                Ok(())
            }
        }
    }

    impl Index for FailingIndex {
        type AttributeValueType = u32;

        fn remove_attribute(&mut self, _handle: Handle, _old_value: &u32) -> Result<(), Box<dyn Error>> {
            self.return_result()
        }

        fn add_attribute(
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

        let handle = universe.add_entity();
        let handle2 = universe.add_entity();
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
