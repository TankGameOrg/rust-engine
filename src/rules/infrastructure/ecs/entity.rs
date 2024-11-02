use std::{
    any::{Any, TypeId},
    collections::HashMap,
    error::Error,
};

use as_any::Downcast;

use crate::rules::infrastructure::RuleError;

use super::attribute::{AnyAttribute, Attribute, AttributeValue};

/// A generic container for storing keys of different types
///
/// ```
/// # use tank_game::rules::infrastructure::ecs::{Attribute, Entity};
/// # static dummy_attribute: Attribute<u32> = Attribute::new("dummy_attribute");
/// #
/// let mut entity = Entity::new();
/// entity.set(&dummy_attribute, 2);
/// assert_eq!(*entity.get(&dummy_attribute)?, 2);
/// #
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Entity {
    attributes: HashMap<&'static dyn AnyAttribute, Box<dyn AttributeValue>>,
}

impl Entity {
    /// Create an empty Entity
    #[inline]
    pub fn new() -> Entity {
        Entity {
            attributes: HashMap::new(),
        }
    }

    /// Get the attribute value from the entity
    pub fn get<T: AttributeValue>(&self, key: &Attribute<T>) -> Result<&T, Box<dyn Error>> {
        match self.attributes.get(key as &dyn AnyAttribute) {
            Some(any) => match any.as_ref().downcast_ref::<T>() {
                Some(value) => Ok(value),
                None => {
                    panic!(
                        "Failed to unwrap attribute '{}' had type {:?} but expected {:?}",
                        key.get_name(),
                        any.type_id(),
                        TypeId::of::<T>()
                    );
                }
            },
            None => Err(Box::new(RuleError::AttributeNotFound {
                name: key.get_name(),
            })),
        }
    }

    /// Store the value of the attribute in the entity
    #[inline]
    pub fn set<T: AttributeValue>(&mut self, key: &'static Attribute<T>, value: T) {
        self.attributes.insert(key, Box::new(value));
    }

    /// Check if this entity has the specified attribute
    #[inline]
    pub fn has(&self, key: &dyn AnyAttribute) -> bool {
        self.attributes.contains_key(key as &dyn AnyAttribute)
    }

    /// Remove an attribute from this entity
    #[inline]
    pub fn remove(&mut self, key: &dyn AnyAttribute) {
        self.attributes.remove(key);
    }

    /// Iterate the attributes stored in the entity
    // TODO: Proper IntoIter
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&dyn AnyAttribute, &dyn AttributeValue)> {
        self.attributes.iter()
            .map(|(attribute, attribute_value)| (*attribute, attribute_value.as_ref()))
    }
}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Entity ")?;
        self.attributes.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use core::panic;

    use crate::rules::infrastructure::{ecs::attribute::DUMMY_ATTRIBUTE, RuleError};

    use super::Entity;

    #[test]
    fn can_get_and_set_basic_attributes() {
        let mut entity = Entity::new();
        entity.set(&DUMMY_ATTRIBUTE, 123);
        assert_eq!(*entity.get(&DUMMY_ATTRIBUTE).unwrap(), 123);
    }

    #[test]
    fn can_check_if_an_attribute_exists() {
        let mut entity = Entity::new();
        assert!(!entity.has(&DUMMY_ATTRIBUTE));
        entity.set(&DUMMY_ATTRIBUTE, 5);
        assert!(entity.has(&DUMMY_ATTRIBUTE));
    }

    #[test]
    fn can_remove_an_attribute() {
        let mut entity = Entity::new();
        entity.set(&DUMMY_ATTRIBUTE, 4);
        assert!(entity.has(&DUMMY_ATTRIBUTE));
        entity.remove(&DUMMY_ATTRIBUTE);
        assert!(!entity.has(&DUMMY_ATTRIBUTE));
    }

    #[test]
    fn getting_a_missing_attribute_returns_error() {
        let entity = Entity::new();

        match entity.get(&DUMMY_ATTRIBUTE) {
            Ok(_) => panic!("Result can't be ok"),
            Err(err) => {
                if let Some(RuleError::AttributeNotFound { name }) = err.downcast_ref::<RuleError>()
                {
                    assert_eq!(*name, "DUMMY_ATTRIBUTE");
                } else {
                    panic!("Error should be AttributeNotFound but got {:?}", err);
                }
            }
        }
    }
}
