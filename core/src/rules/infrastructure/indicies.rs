//! Generic indicies that can be applied to a wide vaiety of attributes

use std::{collections::{HashMap, HashSet}, hash::Hash, marker::PhantomData};

use super::{ecs::{AttributeValue, Handle, Index}, RuleError};

/// An index that finds all entities with a given attribute
/// ```
/// # use tank_game_core::attribute;
/// # use tank_game_core::rules::infrastructure::ecs::{Universe, Handle};
/// # use tank_game_core::rules::infrastructure::indicies::HasAttribute;
/// # use std::vec::Vec;
/// # use tank_game_core::create_entity;
/// attribute!(DummyAttribute: u32, indexed by HasAttribute<u32>);
/// 
/// let mut universe = Universe::new();
/// universe.add_default_index(&DummyAttribute);
/// 
/// // Create an entity with the attribute we want to track
/// let handle = create_entity!(&mut universe, {
///     DummyAttribute = 1
/// })?;
/// 
/// let handles_with_dummy_attr: Vec<Handle> = universe.get_index(&DummyAttribute).gather().collect();
/// #
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct HasAttribute<T: AttributeValue> {
    phantom: PhantomData<T>,
    handles: HashSet<Handle>,
}

impl<T: AttributeValue> HasAttribute<T> {
    pub fn new() -> HasAttribute<T> {
        HasAttribute {
            phantom: PhantomData,
            handles: HashSet::new(),
        }
    }

    /// Get the handles of all of the containers that have the attribute that this index is registed for
    pub fn gather<'index>(&'index self) -> impl Iterator<Item = Handle> + 'index {
        self.handles.iter().cloned()
    }
} 

impl<T: AttributeValue> Default for HasAttribute<T> {
    fn default() -> Self {
        HasAttribute::new()
    }
}

impl<T: AttributeValue> Index for HasAttribute<T> {
    type AttributeValueType = T;

    fn add_attribute(
            &mut self,
            handle: Handle,
            _new_value: &Self::AttributeValueType,
        ) -> Result<(), Box<dyn std::error::Error>> {
        self.handles.insert(handle);
        Ok(())
    }

    fn update_attribute(
            &mut self,
            _handle: Handle,
            _old_value: &Self::AttributeValueType,
            _new_value: &Self::AttributeValueType,
        ) -> Result<(), Box<dyn std::error::Error>> {
        // We only track the existence of our attribute so updates don't matter to us
        Ok(())
    }

    fn remove_attribute(&mut self, handle: Handle, _old_value: &Self::AttributeValueType) -> Result<(), Box<dyn std::error::Error>> {
        self.handles.remove(&handle);
        Ok(())
    }
}

/// Short hand for an attribute value that implements Hash, Eq, and Clone
pub trait HashableAttribute: AttributeValue + Hash + std::cmp::Eq + Clone {}

impl<T: AttributeValue + Hash + std::cmp::Eq + Clone> HashableAttribute for T {}

/// An index for looking up attributes by their value where each value only appears once in a given universe
/// 
/// ```
/// # use tank_game_core::attribute;
/// # use tank_game_core::rules::infrastructure::ecs::{Universe, Handle};
/// # use tank_game_core::rules::infrastructure::indicies::UniqueValueIndex;
/// # use std::vec::Vec;
/// # use tank_game_core::create_entity;
/// attribute!(DummyUnique: u32, indexed by UniqueValueIndex<u32>);
/// 
/// let mut universe = Universe::new();
/// universe.add_default_index(&DummyUnique);
/// 
/// let _ = create_entity!(&mut universe, { DummyUnique = 1 })?;
/// 
/// let _one = universe.get_index(&DummyUnique).get(&1);
///
/// let err = create_entity!(&mut universe, { DummyUnique = 1 });
/// assert!(err.is_err());
/// #
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
/// 
/// If two entities have attributes with the same value that will trigger an error
/// ```
/// # use tank_game_core::attribute;
/// # use tank_game_core::rules::infrastructure::ecs::{Universe, Handle};
/// # use tank_game_core::rules::infrastructure::indicies::UniqueValueIndex;
/// # use std::vec::Vec;
/// # use tank_game_core::create_entity;
/// # attribute!(DummyUnique: u32, indexed by UniqueValueIndex<u32>);
/// #
/// let mut universe = Universe::new();
/// universe.add_default_index(&DummyUnique);
/// 
/// let _ = create_entity!(&mut universe, { DummyUnique = 1 })?;
///
/// // Can't set DummyUnique to 1 because another entity already has that value
/// let _ = create_entity!(&mut universe, { DummyUnique = 1 });
/// #
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct UniqueValueIndex<T: HashableAttribute> {
    by_value: HashMap<T, Handle>,
}

impl<T: HashableAttribute> UniqueValueIndex<T> {
    pub fn new() -> UniqueValueIndex<T> {
        UniqueValueIndex {
            by_value: HashMap::new(),
        }
    }

    /// Find the handle for the attribute with the specified value if one exists
    pub fn get(&self, value: &T) -> Option<Handle> {
        self.by_value.get(&value).cloned()
    }

    /// Collect all handles with this attribute
    pub fn gather<'iter>(&'iter self) -> impl Iterator<Item = Handle> + 'iter {
        self.by_value.values().cloned()
    }
}

impl<T: HashableAttribute> Default for UniqueValueIndex<T> {
    fn default() -> Self {
        UniqueValueIndex::new()
    }
}

impl<T: AttributeValue + Hash + std::cmp::Eq + Clone> Index for UniqueValueIndex<T> {
    type AttributeValueType = T;

    fn add_attribute(
            &mut self,
            handle: Handle,
            new_value: &Self::AttributeValueType,
        ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(existing_handle) = self.by_value.get(new_value) {
            Err(Box::new(RuleError::Generic(format!("The attribute value {:?} is already taken by {:?}", new_value, existing_handle))))
        }
        else {
            self.by_value.insert(new_value.clone(), handle);
            Ok(())
        }
    }

    fn remove_attribute(&mut self, _handle: Handle, old_value: &Self::AttributeValueType) -> Result<(), Box<dyn std::error::Error>> {
        self.by_value.remove(old_value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{attribute, create_entity, rules::infrastructure::ecs::{Handle, Universe}};

    use super::{HasAttribute, UniqueValueIndex};

    attribute!(DummyHas: u32, indexed by HasAttribute<u32>);

    #[test]
    fn test_has_attribute() {
        let mut universe = Universe::new();
        universe.add_default_index(&DummyHas);

        // Create an entity with the attribute we want to track
        let handle = create_entity!(&mut universe, {
            DummyHas = 1
        }).unwrap();

        let _ = universe.add_entity();

        let handles_with_dummy_attr: Vec<Handle> = universe.get_index(&DummyHas).gather().collect();
        assert_eq!(handles_with_dummy_attr.len(), 1);
        assert_eq!(handles_with_dummy_attr[0], handle);
    }

    attribute!(DummyUnique: u32, indexed by UniqueValueIndex<u32>);

    #[test]
    fn test_unique_index() {
        let mut universe = Universe::new();
        universe.add_default_index(&DummyUnique);

        let one = create_entity!(&mut universe, { DummyUnique = 1 }).unwrap();
        let two = create_entity!(&mut universe, { DummyUnique = 2 }).unwrap();

        let index = universe.get_index(&DummyUnique);
        assert_eq!(index.get(&1).unwrap(), one);
        assert_eq!(index.get(&2).unwrap(), two);
        assert_eq!(index.get(&3), None);

        let err = create_entity!(&mut universe, { DummyUnique = 1 });
        assert!(err.is_err());
    }
}