use std::{any, collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};

use as_any::AsAny;

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

pub struct HandleIterator<'iter> {
    iter: Box<dyn Iterator<Item = Handle> + 'iter>,
}

impl<'iter> HandleIterator<'iter> {
    pub fn new(iter: impl Iterator<Item = &'iter Handle> + 'iter) -> HandleIterator<'iter> {
        HandleIterator {
            iter: Box::new(iter.map(|handle| *handle)),
        }
    }
}

impl<'iter> Iterator for HandleIterator<'iter> {
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub trait AttributeStore: AsAny + std::fmt::Debug {
    /// Get a string representing the type of Attribute stored by this store
    fn get_attribute_type_name(&self) -> &'static str;

    /// Get the number of attributes stored by this store
    fn len(&self) -> usize;

    /// Iterate all of the handles the have this store's attribute
    fn iter_handles(&self) -> HandleIterator;

    /// Remove the attribute owned by a handle
    fn remove_attribute(&mut self, handle: Handle);

    /// Get the attribute assosiated with this handle as a generic attribute
    /// 
    /// If the handle does not have an attribute assosiated with it it will panic
    fn get_attribute_generic(&self, handle: Handle) -> &dyn Attribute;
}

#[derive(Debug)]
pub struct GenericAttributeStore<T: Attribute> {
    values: HashMap<Handle, T>,
}

impl<T: Attribute> Default for GenericAttributeStore<T> {
    fn default() -> Self {
        GenericAttributeStore {
            values: HashMap::new(),
        }
    }
}

impl<T: Attribute> GenericAttributeStore<T> {
    /// Get the attribute assosiated with this handle
    /// 
    /// If the handle does not have an attribute assosiated with it it will panic
    pub fn get_attribute(&self, handle: Handle) -> &T {
        self.values.get(&handle).unwrap()
    }

    /// Set the attribute assosiated with this handle
    pub fn set_attribute(&mut self, handle: Handle, value: T) {
        self.values.insert(handle, value);
    }
}

impl<T: Attribute> AttributeStore for GenericAttributeStore<T> {
    fn get_attribute_type_name(&self) -> &'static str {
        any::type_name::<T>()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn iter_handles(&self) -> HandleIterator {
        HandleIterator::new(self.values.keys())
    }

    fn get_attribute_generic(&self, handle: Handle) -> &dyn Attribute {
        self.get_attribute(handle)
    }

    fn remove_attribute(&mut self, handle: Handle) {
        self.values.remove(& handle);
    }
}