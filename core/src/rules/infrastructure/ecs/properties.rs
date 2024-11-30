use std::{any::TypeId, collections::HashMap};

use as_any::{AsAny, Downcast};

pub trait Property: AsAny + std::fmt::Debug {}

#[derive(Debug, Default)]
pub struct Properties {
    properties: HashMap<TypeId, Box<dyn Property>>,
}

impl Properties {
    /// Get a property
    #[inline]
    pub fn get<T: Property>(&self) -> Option<&T> {
        self.properties
            .get(&TypeId::of::<T>())
            .map(|dyn_property| dyn_property.as_ref().downcast_ref::<T>().unwrap())
    }

    /// Check if a property exists
    #[inline]
    pub fn has<T: Property>(&self) -> bool {
        self.properties.contains_key(&TypeId::of::<T>())
    }

    /// Set a property
    ///
    /// Each property can only be set once, subsiquent attempts to set a property will trigger an assert
    #[inline]
    pub fn set<T: Property>(&mut self, property: T) {
        assert!(!self.has::<T>(), "Properies can currently only be set once");
        self.properties
            .insert(TypeId::of::<T>(), Box::new(property));
    }
}
