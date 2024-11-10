use std::{
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
};

use as_any::{AsAny, Downcast};

use crate::rules::infrastructure::RuleError;

use super::AttributeValue;

/// A handle can be used to access and modify an Entity in a Universe
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
#[must_use]
pub struct Handle(usize);

static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0);

impl Handle {
    #[inline]
    pub fn new() -> Handle {
        Handle(NEXT_HANDLE.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for Handle {
    fn default() -> Self {
        Handle::new()
    }
}

/// A type that can optimize searches for entities with a specified attribute
///
/// The Index trait provides a set of methods to update the index when an attribute changes
/// but it does not provide an api for querying the index.  It is assumed that users will downcast
/// the index and call an index specific query API.
pub trait Index: AsAny + Default {
    type AttributeValueType: AttributeValue;

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
    fn remove_attribute(&mut self, handle: Handle, old_value: &Self::AttributeValueType) -> Result<(), Box<dyn Error>>;
}


/// GenericIndex is the internal, boxable, representation of an index
///
/// It allows us to store Indicies with multiple AttributeValue types in the same HashMap
pub trait AnyIndex: AsAny {
    fn add_attribute_dynamic(
        &mut self,
        handle: Handle,
        new_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>>;
    fn update_attribute_dynamic(
        &mut self,
        handle: Handle,
        old_value: &dyn AttributeValue,
        new_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>>;
    fn remove_attribute_dynamic(&mut self, handle: Handle, old_value: &dyn AttributeValue) -> Result<(), Box<dyn Error>>;
}

fn cast_index_value<T: AttributeValue>(value: &dyn AttributeValue) -> Result<&T, Box<dyn Error>> {
    Ok(value
            .downcast_ref()
            .ok_or(Box::new(RuleError::Generic(format!(
                "Failed to cast value to {} from {:?}",
                stringify!(T),
                value.type_id()
            ))))?)
}

impl<F: Index> AnyIndex for F {
    fn add_attribute_dynamic(
            &mut self,
            handle: Handle,
            new_value: &dyn AttributeValue,
        ) -> Result<(), Box<dyn Error>> {
        let new_value = cast_index_value(new_value)?;
        self.add_attribute(handle, new_value)
    }

    fn update_attribute_dynamic(
        &mut self,
        handle: Handle,
        old_value: &dyn AttributeValue,
        new_value: &dyn AttributeValue,
    ) -> Result<(), Box<dyn Error>> {
        let new_value = cast_index_value(new_value)?;
        let old_value = cast_index_value(old_value)?;
        self.update_attribute(handle, old_value, new_value)
    }

    fn remove_attribute_dynamic(&mut self, handle: Handle, old_value: &dyn AttributeValue) -> Result<(), Box<dyn Error>> {
        let old_value = cast_index_value(old_value)?;
        self.remove_attribute(handle, old_value)
    }
}
