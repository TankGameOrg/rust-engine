use std::{any::{Any, TypeId}, collections::HashMap, error::Error};

use as_any::Downcast;

use crate::rules::infrastructure::error::RuleError;

use super::attribute::{Attribute, AttributeValue};

pub struct AttributeContainer {
    attributes: HashMap<&'static str, Box<dyn AttributeValue>>
}

impl AttributeContainer {
    pub fn new() -> AttributeContainer {
        AttributeContainer {
            attributes: HashMap::new(),
        }
    }

    pub fn get<T: AttributeValue>(&self, key: &Attribute<T>) -> Result<&T, Box<dyn Error>> {
        match self.attributes.get(key.get_name()) {
            Some(any) => {
                match any.as_ref().downcast_ref::<T>() {
                    Some(value) => Ok(value),
                    None => {
                        panic!("Failed to unwrap attribute '{}' had type {:?} but expected {:?}", key.get_name(), any.type_id(), TypeId::of::<T>());
                    },
                }
            },
            None => Err(Box::new(RuleError::AttributeNotFound { name: key.get_name() })),
        }
    }

    pub fn set<T: AttributeValue>(&mut self, key: &Attribute<T>, value: T) {
        self.attributes.insert(key.get_name(), Box::new(value));
    }

    pub fn has<T: AttributeValue>(&self, key: &Attribute<T>) -> bool {
        self.attributes.contains_key(key.get_name())
    }

    // TODO: Proper IntoIter
    pub fn iter(&self) -> impl Iterator<Item=(&&'static str, &Box<dyn AttributeValue>)> {
        self.attributes.iter()
    }
}

impl std::fmt::Debug for AttributeContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AttributeContainer ")?;
        self.attributes.fmt(f)
    }
}
