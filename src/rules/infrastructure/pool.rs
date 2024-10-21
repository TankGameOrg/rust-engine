use std::{collections::HashMap, error::Error};

use crate::rules::infrastructure::error::RuleError;

use super::{attribute::AttributeValue, container::AttributeContainer};


/// A handle can be used to access and modify an AttributeContainer in a Pool
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Handle(u32);

static mut NEXT_HANDLE: u32 = 0;

impl Handle {
    #[inline]
    pub(super) fn new() -> Handle {
        Handle(unsafe { NEXT_HANDLE += 1; NEXT_HANDLE })
    }
}

impl AttributeValue for Handle {}

/// A collection of attribute containers that can be queried by their attributes
pub struct Pool {
    containers: HashMap<Handle, AttributeContainer>
}

impl Pool {
    #[inline]
    pub fn new() -> Pool {
        Pool {
            containers: HashMap::new(),
        }
    }

    /// Add an attribute container with an existing handle
    ///
    /// This method exists to allow the AddContainerModification to return a handle when it's created even though the container
    /// itself hasn't been created yet
    #[inline]
    pub(super) fn add_attribute_container_with_handle(&mut self, handle: Handle, container: AttributeContainer) -> Result<(), Box<dyn Error>> {
        if self.containers.contains_key(&handle) {
            let current = self.containers.get(&handle).unwrap();
            return Err(Box::new(RuleError::Generic(format!("The handle {:?} already exists in this pool (current = {:?}, new = {:?}", handle, current, container))))
        }

        self.containers.insert(handle, container);
        Ok(())
    }

    /// Add an attribute container and return a handle that can be used to access it
    #[inline]
    pub fn add_attribute_container(&mut self, container: AttributeContainer) -> Handle {
        let handle = Handle::new();
        self.containers.insert(handle, container);
        handle
    }

    /// Get the attribute container pointed to by a handle
    ///
    /// If the container does not exist we return an error
    #[inline]
    pub fn get_attribute_container(&self, handle: Handle) -> Result<&AttributeContainer, Box<dyn Error>> {
        self.containers.get(&handle)
            .ok_or(Box::new(RuleError::Generic(format!("Attribute container for {:?} does not exist", handle))))
    }

    /// Get a mutable reference to the attribute container pointed to by a haandle
    ///
    /// If the container does not exist we return an error
    #[inline]
    pub(super) fn get_attribute_container_mut(&mut self, handle: Handle) -> Result<&mut AttributeContainer, Box<dyn Error>> {
        self.containers.get_mut(&handle)
            .ok_or(Box::new(RuleError::Generic(format!("Attribute container for {:?} does not exist", handle))))
    }

    pub fn gather<'a>(&'a self, predicate: &'a dyn Fn(&AttributeContainer) -> bool) -> impl Iterator<Item = (Handle, &'a AttributeContainer)> {
        self.containers.iter()
            .filter(|(_, container)| predicate(*container))
            .map(|(handle, container)| (*handle, container))
    }
}

impl std::fmt::Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pool ")?;
        self.containers.fmt(f)
    }
}
