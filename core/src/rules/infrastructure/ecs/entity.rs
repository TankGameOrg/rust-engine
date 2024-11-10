use std::{any::TypeId, collections::HashMap, error::Error};

use as_any::Downcast;

use crate::basic_error;

use super::attribute::{AnyAttribute, Attribute, AttributeValue};

/// A generic container for storing keys of different types
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
    pub fn get<T: AttributeValue>(&self, key: &dyn Attribute<T>) -> Result<&T, Box<dyn Error>> {
        match self.attributes.get(key.as_any_attribute()) {
            Some(any) => match any.as_ref().downcast_ref::<T>() {
                Some(value) => Ok(value),
                None => {
                    panic!(
                        "Failed to unwrap attribute '{}' had type {:?} but expected {:?}",
                        key.get_name(),
                        any.as_ref().type_id(),
                        TypeId::of::<T>()
                    );
                }
            },
            None => Err(basic_error!("Could not find attribute '{}'", key.get_name())),
        }
    }

    /// Store the value of the attribute in the entity
    #[inline]
    pub fn set<T: AttributeValue>(&mut self, key: &'static dyn Attribute<T>, value: T) {
        self.attributes
            .insert(key.as_any_attribute(), Box::new(value));
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
    #[inline]
    pub fn iter(&self) -> AttributeIterator {
        AttributeIterator {
            iter: self.attributes.iter(),
        }
    }
}

impl Default for Entity {
    fn default() -> Self {
        Entity::new()
    }
}

/// An iterator for the attributes in an entity
pub struct AttributeIterator<'iter> {
    iter: std::collections::hash_map::Iter<'iter, &'iter dyn AnyAttribute, Box<dyn AttributeValue>>,
}

impl<'iter> Iterator for AttributeIterator<'iter> {
    type Item = (&'iter dyn AnyAttribute, &'iter dyn AttributeValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(attribute, value)| (*attribute, value.as_ref()))
    }
}

impl<'iter> IntoIterator for &'iter Entity {
    type Item = (&'iter dyn AnyAttribute, &'iter dyn AttributeValue);
    type IntoIter = AttributeIterator<'iter>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
    use crate::rules::infrastructure::ecs::attribute::DummyAttribute;

    use super::Entity;

    #[test]
    fn can_get_and_set_basic_attributes() {
        let mut entity = Entity::new();
        entity.set(&DummyAttribute, 123);
        assert_eq!(*entity.get(&DummyAttribute).unwrap(), 123);
    }

    #[test]
    fn can_check_if_an_attribute_exists() {
        let mut entity = Entity::new();
        assert!(!entity.has(&DummyAttribute));
        entity.set(&DummyAttribute, 5);
        assert!(entity.has(&DummyAttribute));
    }

    #[test]
    fn can_remove_an_attribute() {
        let mut entity = Entity::new();
        entity.set(&DummyAttribute, 4);
        assert!(entity.has(&DummyAttribute));
        entity.remove(&DummyAttribute);
        assert!(!entity.has(&DummyAttribute));
    }

    #[test]
    fn can_iterate_attributes() {
        // as_any::Downcast overrides Error.downcast_ref in getting_a_missing_attribute_returns_error which breaks the test
        use as_any::Downcast;

        let mut entity = Entity::new();
        entity.set(&DummyAttribute, 4);

        let mut found_value: u32 = 0;

        for (attribute, value) in &entity {
            assert_eq!(attribute.get_name(), "DummyAttribute");
            found_value = *value.downcast_ref().unwrap();
        }

        assert_eq!(found_value, 4);
    }

    #[test]
    fn getting_a_missing_attribute_returns_error() {
        let entity = Entity::new();
        assert!(entity.get(&DummyAttribute).is_err());
    }
}
