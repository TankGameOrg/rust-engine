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

    /// Remove an attribute from this container
    #[inline]
    pub fn remove<T: AttributeValue>(&mut self, key: &Attribute<T>) {
        self.attributes.remove(&key.get_name());
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

#[cfg(test)]
mod test {
    use core::panic;

    use crate::rules::infrastructure::{attribute::DUMMY_ATTRIBUTE, error::RuleError};

    use super::AttributeContainer;

    #[test]
    fn can_get_and_set_basic_attributes() {
        let mut container = AttributeContainer::new();
        container.set(&DUMMY_ATTRIBUTE, 123);
        assert_eq!(*container.get(&DUMMY_ATTRIBUTE).unwrap(), 123);
    }

    #[test]
    fn can_check_if_an_attribute_exists() {
        let mut container = AttributeContainer::new();
        assert!(!container.has(&DUMMY_ATTRIBUTE));
        container.set(&DUMMY_ATTRIBUTE, 5);
        assert!(container.has(&DUMMY_ATTRIBUTE));
    }

    #[test]
    fn can_remove_an_attribute() {
        let mut container = AttributeContainer::new();
        container.set(&DUMMY_ATTRIBUTE, 4);
        assert!(container.has(&DUMMY_ATTRIBUTE));
        container.remove(&DUMMY_ATTRIBUTE);
        assert!(!container.has(&DUMMY_ATTRIBUTE));
    }

    #[test]
    fn getting_a_missing_attribute_returns_error() {
        let container = AttributeContainer::new();

        match container.get(&DUMMY_ATTRIBUTE) {
            Ok(_) => panic!("Result can't be ok"),
            Err(err) => {
                if let Some(RuleError::AttributeNotFound { name }) = err.downcast_ref::<RuleError>() {
                    assert_eq!(*name, "DUMMY_ATTRIBUTE");
                }
                else {
                    panic!("Error should be AttributeNotFound but got {:?}", err);
                }
            }
        }
    }
}
