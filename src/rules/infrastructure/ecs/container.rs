use std::{any::{Any, TypeId}, collections::HashMap, error::Error};

use crate::rules::infrastructure::error::RuleError;

use super::attribute::Attribute;

#[derive(Debug)]
pub struct AttributeContainer {
    attributes: HashMap<&'static str, Box<dyn Any>>
}

impl AttributeContainer {
    pub fn new() -> AttributeContainer {
        AttributeContainer {
            attributes: HashMap::new(),
        }
    }

    pub fn get<T: 'static>(&self, key: &Attribute<T>) -> Result<&T, Box<dyn Error>> {
        match self.attributes.get(key.get_name()) {
            Some(any) => {
                match any.downcast_ref::<T>() {
                    Some(value) => Ok(value),
                    None => {
                        panic!("Failed to unwrap attribute '{}' had type {:?} but expected {:?}", key.get_name(), any.type_id(), TypeId::of::<T>());
                    },
                }
            },
            None => Err(Box::new(RuleError::AttributeNotFound { name: key.get_name() })),
        }
    }

    pub fn set<T: 'static>(&mut self, key: &Attribute<T>, value: T) {
        self.attributes.insert(key.get_name(), Box::new(value));
    }

    pub fn has<T: 'static>(&self, key: &Attribute<T>) -> bool {
        self.attributes.contains_key(key.get_name())
    }

    // TODO: Proper IntoIter
    pub fn iter(&self) -> impl Iterator<Item=(&&'static str, &Box<dyn Any>)> {
        self.attributes.iter()
    }
}

