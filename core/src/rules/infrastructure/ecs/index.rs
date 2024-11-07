use std::{
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
};

use as_any::AsAny;

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
    fn remove_attribute(&mut self, handle: Handle) -> Result<(), Box<dyn Error>>;
}
