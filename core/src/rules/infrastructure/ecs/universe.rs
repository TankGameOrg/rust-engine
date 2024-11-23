use std::{any, collections::HashMap, error::Error};

use as_any::Downcast;

use crate::basic_error;

use super::{
    attribute::{AnyAttribute, Attribute, AttributeId, AttributeValue, FlagAttribute}, signature::{EntitySignature, Signature}, store::{AttributeStore, GenericAttributeStore, Handle, HandleIterator}
};

/// A collection of entities that can be queried by their attributes
pub struct Universe {
    entities: HashMap<Handle, EntitySignature>,
    attribute_stores: HashMap<AttributeId, Box<dyn AttributeStore>>,
}

impl Default for Universe {
    fn default() -> Self {
        Self::new()
    }
}

impl Universe {
    #[inline]
    pub fn new() -> Universe {
        Universe {
            entities: HashMap::new(),
            attribute_stores: HashMap::new(),
        }
    }

    /// Add an entity and create a new handle
    pub fn add_entity(&mut self) -> Handle {
        let handle = Handle::new();
        self.entities.insert(handle, EntitySignature::default());
        handle
    }

    /// Add an Entity with an existing handle
    ///
    /// Most users should call add_entity() and use the handle it gives you
    pub(in crate::rules::infrastructure) fn add_entity_with_handle(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
        if self.entities.contains_key(&handle) {
            let current = self.entities.get(&handle).unwrap();
            return Err(basic_error!(
                "The handle {:?} already exists in this universe (current = {:?})",
                handle, current
            ));
        }

        self.entities.insert(handle, EntitySignature::default());
        Ok(())
    }

    fn get_signature(&self, handle: &Handle) -> Result<&EntitySignature, Box<dyn Error>> {
        self.entities.get(&handle)
            .ok_or(basic_error!("There is no entity for handle {:?}", handle))
    }

    fn get_signature_mut(&mut self, handle: &Handle) -> Result<&mut EntitySignature, Box<dyn Error>> {
        self.entities.get_mut(&handle)
            .ok_or(basic_error!("There is no entity for handle {:?}", handle))
    }

    /// Get an attribute's value from an entity
    ///
    /// If the entity doesn't exist or it doesn't exist we return an error
    #[inline]
    pub fn get_attribute<T: AttributeValue>(
        &self,
        handle: Handle,
        key: &Attribute<T>,
    ) -> Result<&T, Box<dyn Error>> {
        let attribute_id = key.get_attribute_id();
        let signature = self.get_signature(&handle)?;
        if !signature.has(attribute_id) {
            return Err(basic_error!("Entity {:?} does not have the attribute {} (signature = {:?})", handle, attribute_id, signature));
        }

        let store = self.attribute_stores.get(&attribute_id).unwrap();
        let store_type_name = store.get_attribute_type_name();

        let store: &GenericAttributeStore<T> = store.as_ref()
            .downcast_ref()
            .ok_or(basic_error!("Attribute store for {} should be of type {} but was {}", key.get_name(), any::type_name::<T>(), store_type_name))?;

        Ok(store.get_attribute(handle))
    }

    /// Set an attribute's value for an entity
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn set_attribute<T: AttributeValue>(
        &mut self,
        handle: Handle,
        key: &Attribute<T>,
        value: T,
    ) -> Result<(), Box<dyn Error>> {
        let attribute_id = key.get_attribute_id();
        self.get_signature_mut(&handle)?.add(attribute_id);

        if !self.attribute_stores.contains_key(&attribute_id) {
            self.attribute_stores.insert(attribute_id, Box::new(GenericAttributeStore::<T>::default()));
        }

        let store = self.attribute_stores.get_mut(&attribute_id).unwrap();
        let store_type_name = store.get_attribute_type_name();

        let store: &mut GenericAttributeStore<T> = store.as_mut()
            .downcast_mut()
            .ok_or(basic_error!("Attribute store for {} should be of type {} but was {}", key.get_name(), any::type_name::<T>(), store_type_name))?;

        store.set_attribute(handle, value);
        Ok(())
    }

    /// Set a flag attribute
    /// 
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn set_flag_attribute(
        &mut self,
        handle: Handle,
        key: &FlagAttribute
    ) -> Result<(), Box<dyn Error>> {
        let attribute_id = key.get_attribute_id();
        self.get_signature_mut(&handle)?.add(attribute_id);
        Ok(())
    }

    /// Remove an attribute from an entity
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn remove_attribute(
        &mut self,
        handle: Handle,
        key: &dyn AnyAttribute,
    ) -> Result<(), Box<dyn Error>> {
        let attribute_id = key.get_attribute_id();
        let signature = self.get_signature_mut(&handle)?;

        if !signature.has(attribute_id) {
            return Ok(());
        }
        
        signature.remove(attribute_id);

        if key.is_flag() {
            return Ok(());
        }

        let store = self.attribute_stores.get_mut(&attribute_id).unwrap();
        store.remove_attribute(handle);

        Ok(())
    }

    /// Check an entity has an attribute
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn has_attribute(
        &self,
        handle: Handle,
        key: &dyn AnyAttribute,
    ) -> Result<bool, Box<dyn Error>> {
        Ok(self.get_signature(&handle)?.has(key.get_attribute_id()))
    }

    // TODO: Implement me
    /// Iterate the attributes on an entity
    // #[inline]
    // pub fn iter_attributes(
    //     &self,
    //     handle: Handle,
    // ) -> Result<impl Iterator<Item = (&dyn AnyAttribute, &dyn AttributeValue)>, Box<dyn Error>>
    // {
    //     Err(basic_error!("TODO"))
    // }

    /// Remove a entity from a universe
    ///
    /// If the handle does not exist return an error
    pub fn remove_entity(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
        let optional_entity_sig = self.entities.remove(&handle);

        match optional_entity_sig {
            None => Err(basic_error!(
                "The handle {:?} does not reference a valid entity",
                handle
            )),
            Some(entity_sig) => {
                for attribute_id in entity_sig.iter_attribute_ids() {
                    if let Some(store) = self.attribute_stores.get_mut(&attribute_id) {
                        store.remove_attribute(handle);
                    }
                }

                Ok(())
            }
        }
    }

    pub fn is_match(&self, handle: Handle, signature: Signature) -> bool {
        signature.is_match(self.entities.get(&handle).unwrap())
    }

    /// Filter all of the entities in the universe and return an iterator to the ones that match
    pub fn gather<'iter>(
        &'iter self,
        signature: Signature,
    ) -> impl Iterator<Item = Handle> + 'iter {
        let mut handle_iter = HandleIterator::new(self.entities.keys());
        let mut length = self.entities.len();

        for attribute_id in signature.iter_attribute_ids() {
            if let Some(store) = self.attribute_stores.get(&attribute_id) {
                if store.len() < length {
                    length = store.len();
                    handle_iter = store.iter_handles();
                }
            }
        }

        handle_iter.filter(move |handle| self.is_match(*handle, signature))
    }
}

impl std::fmt::Debug for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Universe ")?;
        self.entities.fmt(f)
    }
}


/// Create an entity with the specified attributes
///
/// ```
/// # use tank_game_core::attribute;
/// # use std::error::Error;
/// # use tank_game_core::rules::infrastructure::ecs::Universe;
/// # use tank_game_core::create_entity;
/// # attribute!(DummyAttribute: u32);
/// # attribute!(DummyAttribute2: u32);
/// #
/// let mut universe = Universe::new();
/// let new_handle = create_entity!(&mut universe, {
///     DummyAttribute = 3,
///     DummyAttribute2 = 4
/// })?;
/// #
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! create_entity {
    ($universe:expr, $($token:tt)*) => {
        {
            use $crate::modify_entity;

            let universe: &mut $crate::rules::infrastructure::ecs::Universe = $universe;

            let handle = universe.add_entity();

            modify_entity!(universe, handle, $($token)*)
                .map(|()| handle)
        }
    };
}

/// Set the specified attributes on an existing entity
///
/// ```
/// # use tank_game_core::attribute;
/// # use std::error::Error;
/// # use tank_game_core::rules::infrastructure::ecs::{Attribute, Universe};
/// # use tank_game_core::{create_entity, modify_entity};
/// # attribute!(DummyAttribute: u32);
/// # attribute!(DummyAttribute2: u32);
/// #
/// let mut universe = Universe::new();
/// # let dummy_handle = create_entity!(&mut universe, { DummyAttribute = 3 })?;
/// modify_entity!(&mut universe, dummy_handle, {
///     DummyAttribute = 3,
///     DummyAttribute2 = 4
/// })?;
/// #
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! modify_entity {
    ($universe:expr, $handle:ident, { $($attribute:ident = $value:expr),+ }) => {
        {
            let universe: &mut $crate::rules::infrastructure::ecs::Universe = $universe;
            let handle: $crate::rules::infrastructure::ecs::Handle = $handle;
            let mut result = Ok(());

            $(
                if result.is_ok() {
                    result = universe.set_attribute(handle, &$attribute, $value);
                }
            )+

            result
        }
    };
}


#[cfg(test)]
mod test {
    use crate::{attribute, rules::infrastructure::ecs::attribute::DummyAttribute, signature};

    use super::*;

    #[test]
    fn can_modify_and_retrieve_entities() {
        let mut universe = Universe::new();
        let handle = universe.add_entity();
        universe.set_attribute(handle, &DummyAttribute, 2).unwrap();

        assert_eq!(*universe.get_attribute(handle, &DummyAttribute).unwrap(), 2);
    }

    #[test]
    fn can_add_a_entity_with_an_existing_handle() {
        let mut universe = Universe::new();
        let handle = universe.add_entity();

        let error = universe.add_entity_with_handle(handle);
        assert!(error.is_err());
    }

    attribute!(flag DummyAttribute2);

    #[test]
    fn can_gather_entities() {
        let mut universe = Universe::new();
        let first_handle = universe.add_entity();
        universe
            .set_attribute(first_handle, &DummyAttribute, 2)
            .unwrap();

        let second_handle = universe.add_entity();
        universe
            .set_attribute(second_handle, &DummyAttribute, 1)
            .unwrap();

            universe.set_flag_attribute(second_handle, &DummyAttribute2).unwrap();

        let _ = universe.add_entity();

        // Gather one of the entities
        let one: Vec<Handle> = universe
            .gather(signature!(DummyAttribute2))
            .collect();

        assert_eq!(one.len(), 1);
        assert_eq!(one[0], second_handle);

        // Gather both of the ones with attributes
        let two: Vec<Handle> = universe
            .gather(signature!(DummyAttribute))
            .collect();

        println!("{:?} - {:?}, {:?}", two, first_handle, second_handle);
        assert_eq!(two.len(), 2);
        assert!(two.contains(&first_handle));
        assert!(two.contains(&second_handle));
    }
}
