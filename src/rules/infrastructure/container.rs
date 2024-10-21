use std::{any::{Any, TypeId}, collections::HashMap, error::Error};

use as_any::Downcast;

use super::error::RuleError;
use super::attribute::{Attribute, AttributeValue};

/// A generic container for storing keys of different types
///
/// ```
/// # use tank_game::rules::infrastructure::attribute::Attribute;
/// # use tank_game::rules::infrastructure::container::AttributeContainer;
/// # let dummy_attribute = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut container = AttributeContainer::new();
/// container.set(&dummy_attribute, 2);
/// assert_eq!(*container.get(&dummy_attribute)?, 2);
/// #
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct AttributeContainer {
    attributes: HashMap<&'static str, Box<dyn AttributeValue>>
}

impl AttributeContainer {
    /// Create an empty attribute container
    #[inline]
    pub fn new() -> AttributeContainer {
        AttributeContainer {
            attributes: HashMap::new(),
        }
    }

    /// Get the attribute value from the container
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

    /// Store the value of the attribute in the container
    #[inline]
    pub fn set<T: AttributeValue>(&mut self, key: &Attribute<T>, value: T) {
        self.attributes.insert(key.get_name(), Box::new(value));
    }

    /// Check if this container has the specified attribute
    #[inline]
    pub fn has<T: AttributeValue>(&self, key: &Attribute<T>) -> bool {
        self.attributes.contains_key(key.get_name())
    }

    /// Iterate the attributes stored in the container
    // TODO: Proper IntoIter
    #[inline]
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
