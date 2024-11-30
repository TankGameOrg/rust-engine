use std::{any::{type_name, Any}, error::Error};

use as_any::AsAny;

use crate::basic_error;

use super::{store::DefaultAttributeStore, AttributeStore, Properties};

/// The common ancestor for all attribute values
pub trait Attribute: std::fmt::Debug + Send + Sync + AsAny {
    fn create_store(&self, _properties: &Properties) -> Box<dyn AttributeStore> where Self: Sized {
        Box::new(DefaultAttributeStore::<Self>::default())
    }

    /// Get a human readable name for this attribute
    fn get_display_name(&self) -> &'static str {
        let resource_name= type_name::<Self>();
        let start_index = resource_name.rfind(":")
            .map(|index| index + 1)
            .unwrap_or(0);

        &resource_name[start_index..]
    }
}

// A basic attribute for writing unit tests
#[cfg(test)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DummyAttribute(pub u32);

#[cfg(test)]
impl Attribute for DummyAttribute {}

/// Like `Box<dyn Attribute>` except you can downcast and take ownership of the concrete attribute
pub struct BoxedAttribute(Box<dyn Any>);

impl BoxedAttribute {
    pub fn new(attribute: impl Attribute) -> BoxedAttribute {
        BoxedAttribute(Box::new(attribute))
    }

    pub fn downcast<T: Attribute>(self) -> Result<T, Box<dyn Error>> {
        let value_name = self.0.as_ref().type_id();

        match self.0.downcast() {
            Err(_) => Err(basic_error!(
                "Expected attribute of type {} but got {:?}",
                type_name::<T>(),
                value_name
            )),
            Ok(value) => Ok(*value),
        }
    }
}
