use std::{collections::HashMap, error::Error};

use crate::rules::infrastructure::error::RuleError;

use super::{attribute::AttributeValue, container::AttributeContainer};


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

    #[inline]
    pub(super) fn add_attribute_container_with_handle(&mut self, handle: Handle, container: AttributeContainer) {
        self.containers.insert(handle, container);
    }

    #[inline]
    pub fn add_attribute_container(&mut self, container: AttributeContainer) -> Handle {
        let handle = Handle::new();
        self.add_attribute_container_with_handle(handle, container);
        handle
    }

    #[inline]
    pub fn get_attribute_container(&self, handle: Handle) -> Result<&AttributeContainer, Box<dyn Error>> {
        self.containers.get(&handle)
            .ok_or(Box::new(RuleError::Generic(format!("Attribute container for {:?} does not exist", handle))))
    }

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