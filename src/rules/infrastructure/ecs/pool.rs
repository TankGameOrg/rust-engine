use std::{collections::HashMap, error::Error};

use crate::rules::infrastructure::error::RuleError;

use super::container::AttributeContainer;


#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Handle {
    handle: u32,
}

static mut NEXT_HANDLE: u32 = 0;

impl Handle {
    pub(super) fn new() -> Handle {
        Handle {
            handle: unsafe { NEXT_HANDLE += 1; NEXT_HANDLE },
        }
    }
}

#[derive(Debug)]
pub struct Pool {
    containers: HashMap<Handle, AttributeContainer>
}

impl Pool {
    pub fn new() -> Pool {
        Pool {
            containers: HashMap::new(),
        }
    }

    pub(super) fn add_attribute_container(&mut self, handle: Handle, container: AttributeContainer) {
        self.containers.insert(handle, container);
    }

    pub fn get_attribute_container(&self, handle: Handle) -> Result<&AttributeContainer, Box<dyn Error>> {
        self.containers.get(&handle)
            .ok_or(Box::new(RuleError::Generic(format!("Attribute container for {:?} does not exist", handle))))
    }

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
