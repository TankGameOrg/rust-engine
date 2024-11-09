//! Generic indicies that can be applied to a wide vaiety of attributes

use std::{collections::HashSet, marker::PhantomData};

use super::ecs::{AttributeValue, Handle, Index};

/// Collect all of the entities that have this attribute
/// ```
/// # use tank_game_core::attribute;
/// # use tank_game_core::rules::infrastructure::ecs::{Universe, Handle};
/// # use tank_game_core::rules::infrastructure::indicies::HasAttribute;
/// # use std::vec::Vec;
/// attribute!(DummyAttribute: u32, indexed by HasAttribute<u32>);
/// 
/// let mut universe = Universe::new();
/// universe.add_index(&DummyAttribute, Default::default());
/// 
/// // Create an entity with the attribute we want to track
/// let handle = Handle::new();
/// universe.add_entity(handle);
/// universe.set_attribute(handle, &DummyAttribute, 1);
/// 
/// let _handles_with_dummy_attr: Vec<Handle> = universe.get_index(&DummyAttribute).gather().collect();
/// #
/// # Ok::<(), Box<dyn std::error::Error>>(());
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