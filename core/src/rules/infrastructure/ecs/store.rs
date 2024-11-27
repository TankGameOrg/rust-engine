use std::{
    collections::HashMap,
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::Attribute;

/// A handle can be used to access and modify an Entity in a Universe
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
#[must_use]
pub struct Handle(usize);

static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0);

impl Attribute for Handle {}

impl Handle {
    pub(super) fn new() -> Handle {
        Handle(NEXT_HANDLE.fetch_add(1, Ordering::Relaxed))
    }
}

/// A wrapper for any iterator that returns a Handle
pub struct HandleIterator<'iter> {
    iter: Option<Box<dyn Iterator<Item = Handle> + 'iter>>,
}

impl<'iter> HandleIterator<'iter> {
    pub fn new(iter: impl Iterator<Item = &'iter Handle> + 'iter) -> HandleIterator<'iter> {
        HandleIterator {
            iter: Some(Box::new(iter.map(|handle| *handle))),
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

pub trait AttributeStore: std::fmt::Debug + 'static {
    type StoredAttribute: Attribute;

    /// Get the number of attributes stored by this store
    fn len(&self) -> usize;

    /// Iterate all of the handles the have this store's attribute
    fn iter_handles(&self) -> HandleIterator;

    /// Remove the attribute owned by a handle
    fn remove_attribute(&mut self, handle: Handle);

    /// Get the attribute assosiated with this handle as a generic attribute
    ///
    /// If the handle does not have an attribute assosiated with it it will panic
    fn get_attribute(&self, handle: Handle) -> &Self::StoredAttribute;

    /// Set the attribute assosiated with this handle
    fn set_attribute(
        &mut self,
        handle: Handle,
        value: Self::StoredAttribute,
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
    type StoredAttribute = T;

    fn len(&self) -> usize {
        self.values.len()
    }

    fn iter_handles(&self) -> HandleIterator {
        HandleIterator::new(self.values.keys())
    }

    fn get_attribute(&self, handle: Handle) -> &Self::StoredAttribute {
        self.values.get(&handle).unwrap()
    }

    fn remove_attribute(&mut self, handle: Handle) {
        self.values.remove(&handle);
    }

    fn set_attribute(
        &mut self,
        handle: Handle,
        value: Self::StoredAttribute,
    ) -> Result<(), Box<dyn Error>> {
        self.values.insert(handle, value);
        Ok(())
    }
}

pub trait Query {
    type StoredAttribute: Attribute;
    type Store: AttributeStore<StoredAttribute = Self::StoredAttribute> + Sized;

    fn query<'iter>(&'iter self, store: &'iter Self::Store) -> HandleIterator<'iter>;
}
