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

    pub fn visit_all(&self, visitor: &dyn Fn(&'static str, &dyn Any) -> Result<(), Box<dyn Error>>) -> Result<(), Box<dyn Error>> {
        for (attribute, value) in &self.attributes {
            (visitor)(attribute, value.as_ref())?;
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! make_visitor {
    ($(($var_attribute_name:ident, $var_name:ident: $attribute_type:ty) => $code:expr),+) => {
        |attribute_name: &'static str, any_attribute_value: &dyn std::any::Any| -> Result<(), Box<dyn Error>> {
            let attribute_type_id = any_attribute_value.type_id();
            let mut attribute_value_handled = false;

            $(
                if attribute_type_id == std::any::TypeId::of::<$attribute_type>() {
                    let $var_attribute_name = attribute_name;
                    attribute_value_handled = true;

                    match any_attribute_value.downcast_ref::<$attribute_type>() {
                        Some($var_name) => $code,
                        None => panic!("Not allowed"),
                    }
                }
            )+

            if !attribute_value_handled {
                Err(Box::new(crate::rules::infrastructure::error::RuleError::Generic(format!("No case found for type: {:?}", attribute_type_id))))
            } else {
                Ok(())
            }
        }
    };
}
