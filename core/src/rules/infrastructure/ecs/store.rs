use std::{
    collections::HashMap,
    error::Error
};

use as_any::AsAny;
use uuid::Uuid;

use super::{attribute::BoxedAttribute, Attribute};

/// A handle can be used to access and modify an Entity in a Universe
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
#[must_use]
pub struct Handle(Uuid);

impl Attribute for Handle {}

impl Handle {
    pub(super) fn new() -> Handle {
        Handle(Uuid::new_v4())
    }
}

/// A wrapper for any iterator that returns a Handle
pub struct HandleIterator<'iter> {
    iter: Option<Box<dyn Iterator<Item = Handle> + 'iter>>,
}

impl<'iter> HandleIterator<'iter> {
    pub fn new(iter: impl Iterator<Item = Handle> + 'iter) -> HandleIterator<'iter> {
        HandleIterator {
            iter: Some(Box::new(iter)),
        }
    }

    pub fn empty() -> HandleIterator<'iter> {
        HandleIterator { iter: None }
    }
}

impl<'iter> Iterator for HandleIterator<'iter> {
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.as_mut()?.next()
    }
}

pub trait AttributeStore: std::fmt::Debug + AsAny {
    /// Get the number of attributes stored by this store
    fn len(&self) -> usize;

    /// Check if an attribute store has any elements
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate all of the handles the have this store's attribute
    fn iter_handles(&self) -> HandleIterator;

    /// Remove the attribute owned by a handle
    fn remove_attribute(&mut self, handle: Handle);

    /// Get the attribute assosiated with this handle as a generic attribute
    ///
    /// If the handle does not have an attribute assosiated with it it will panic
    fn get_attribute(&self, handle: Handle) -> &dyn Attribute;

    /// Set the attribute assosiated with this handle
    fn set_attribute(
        &mut self,
        handle: Handle,
        value: BoxedAttribute,
    ) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug)]
pub struct DefaultAttributeStore<T: Attribute> {
    values: HashMap<Handle, T>,
}

impl<T: Attribute> Default for DefaultAttributeStore<T> {
    fn default() -> Self {
        DefaultAttributeStore {
            values: HashMap::new(),
        }
    }
}

impl<T: Attribute> AttributeStore for DefaultAttributeStore<T> {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn iter_handles(&self) -> HandleIterator {
        HandleIterator::new(self.values.keys().cloned())
    }

    fn get_attribute(&self, handle: Handle) -> &dyn Attribute {
        self.values.get(&handle).unwrap()
    }

    fn remove_attribute(&mut self, handle: Handle) {
        self.values.remove(&handle);
    }

    fn set_attribute(
        &mut self,
        handle: Handle,
        value: BoxedAttribute,
    ) -> Result<(), Box<dyn Error>> {
        self.values.insert(handle, value.downcast()?);
        Ok(())
    }
}

/// A type that can be used to find zero or more [EntityRef] in a Universe based on exactly one of their attributes
///
/// [EntityRef]: crate::rules::infrastructure::ecs::EntityRef
pub trait Query {
    type StoredAttribute: Attribute;

    fn query<'iter>(&'iter self, store: &'iter dyn AttributeStore) -> HandleIterator<'iter>;
}

/// A type that can be used to find zero or one [EntityRef] in a Universe based on exactly one of their attributes
///
/// [EntityRef]: crate::rules::infrastructure::ecs::EntityRef
pub trait QueryOne {
    type StoredAttribute: Attribute;

    fn query_one(&self, store: &dyn AttributeStore) -> Option<Handle>;
}
