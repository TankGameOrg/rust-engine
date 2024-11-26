use std::{any, collections::HashMap, error::Error};

use as_any::Downcast;

use crate::basic_error;

use super::{
    attribute::Attribute, signature::{AttributeId, AttributeIdMap, EntitySignature, Signature}, store::{AttributeStore, GenericAttributeStore, Handle, HandleIterator}
};

/// A collection of entities that can be queried by their attributes
#[derive(Debug, Default)]
pub struct Universe {
    id_map: AttributeIdMap,
    entities: HashMap<Handle, EntitySignature>,
    stores: HashMap<AttributeId, Box<dyn AttributeStore>>,
}

impl Universe {
    /// Add an entity and create a new handle
    pub fn add_entity(&mut self) -> Handle {
        let handle = Handle::new();
        self.entities.insert(handle, EntitySignature::default());
        handle
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
    pub fn get_attribute<T: Attribute>(
        &self,
        handle: Handle,
    ) -> Result<&T, Box<dyn Error>> {
        let attribute_id = self.id_map.get_id::<T>();
        let signature = self.get_signature(&handle)?;
        if attribute_id.is_none() || !signature.has(attribute_id.unwrap()) {
            return Err(basic_error!("Entity {:?} does not have the attribute {:?} (signature = {:?})", handle, attribute_id, signature));
        }

        let attribute_id = attribute_id.unwrap();

        let store = self.stores.get(&attribute_id).unwrap();
        let store_type_name = store.get_attribute_type_name();

        let store: &GenericAttributeStore<T> = store.as_ref()
            .downcast_ref()
            .ok_or(basic_error!("Attribute store for {} but was {}", any::type_name::<T>(), store_type_name))?;

        Ok(store.get_attribute(handle))
    }

    /// Set an attribute's value for an entity
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn set_attribute<T: Attribute>(
        &mut self,
        handle: Handle,
        value: T,
    ) -> Result<(), Box<dyn Error>> {
        let attribute_id = self.id_map.get_or_assign_id::<T>();
        self.get_signature_mut(&handle)?.add(attribute_id);

        if !self.stores.contains_key(&attribute_id) {
            self.stores.insert(attribute_id, Box::new(GenericAttributeStore::<T>::default()));
        }

        let store = self.stores.get_mut(&attribute_id).unwrap();
        let store_type_name = store.get_attribute_type_name();

        let store: &mut GenericAttributeStore<T> = store.as_mut()
            .downcast_mut()
            .ok_or(basic_error!("Attribute store for {} but was {}", any::type_name::<T>(), store_type_name))?;

        store.set_attribute(handle, value);
        Ok(())
    }

    /// Remove an attribute from an entity
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn remove_attribute<T: Attribute>(
        &mut self,
        handle: Handle
    ) -> Result<(), Box<dyn Error>> {
        let attribute_id = self.id_map.get_id::<T>();
        if let None = attribute_id {
            return Ok(());
        }

        let attribute_id = attribute_id.unwrap();
        let signature = self.get_signature_mut(&handle)?;

        if !signature.has(attribute_id) {
            return Ok(());
        }
        
        signature.remove(attribute_id);

        let attribute_data = self.stores.get_mut(&attribute_id).unwrap();
        attribute_data.remove_attribute(handle);

        Ok(())
    }

    /// Check an entity has an attribute
    ///
    /// If the entity doesn't exist we return an error
    #[inline]
    pub fn has_attribute<T: Attribute>(
        &self,
        handle: Handle
    ) -> Result<bool, Box<dyn Error>> {
        match self.id_map.get_id::<T>() {
            Some(attribute_id) => Ok(self.get_signature(&handle)?.has(attribute_id)),
            None => Ok(false),
        }
    }

    /// Iterate the attributes on an entity
    #[inline]
    pub fn iter_attributes(
        &self,
        handle: Handle,
    ) -> Result<impl Iterator<Item = &dyn Attribute>, Box<dyn Error>>
    {
        let signature = self.get_signature(&handle)?;

        Ok(signature.iter_attribute_ids()
            .map(move |attribute_id| {
                let store = self.stores.get(&attribute_id).unwrap();
                store.get_attribute_generic(handle)
            }))
    }

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
                    if let Some(store) = self.stores.get_mut(&attribute_id) {
                        store.remove_attribute(handle);
                    }
                }

                Ok(())
            }
        }
    }

    pub fn is_match(&self, handle: Handle, signature: Signature) -> bool {
        match signature.compile(&self.id_map) {
            Some(compiled) => compiled.is_match(self.entities.get(&handle).unwrap()),
            None => false,
        }
    }

    /// Filter all of the entities in the universe and return an iterator to the ones that match
    pub fn gather<'iter>(
        &'iter self,
        signature: Signature,
    ) -> impl Iterator<Item = Handle> + 'iter {
        let mut handle_iter = HandleIterator::new(self.entities.keys());
        let mut length = self.entities.len();

        for attribute_id in signature.iter_included(&self.id_map) {
            if let Some(store) = self.stores.get(&attribute_id) {
                if store.len() < length {
                    length = store.len();
                    handle_iter = store.iter_handles();
                }
            }
        }

        let compiled_signature = signature.compile(&self.id_map);
        handle_iter.filter(move |handle| {
            match &compiled_signature {
                Some(compiled) => compiled.is_match(self.entities.get(handle).unwrap()),
                None => false,
            }
        })
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
    ($universe:expr, $handle:ident, { $($attribute:ty = $value:expr),+ }) => {
        {
            let universe: &mut $crate::rules::infrastructure::ecs::Universe = $universe;
            let handle: $crate::rules::infrastructure::ecs::Handle = $handle;
            let mut result = Ok(());

            $(
                if result.is_ok() {
                    result = universe.set_attribute::<$attribute>(handle, $value);
                }
            )+

            result
        }
    };
}


#[cfg(test)]
mod test {
    use crate::{rules::infrastructure::ecs::attribute::DummyAttribute, signature};

    use super::*;

    #[test]
    fn can_modify_and_retrieve_entities() {
        let mut universe = Universe::default();
        let handle = universe.add_entity();
        universe.set_attribute(handle, DummyAttribute(2)).unwrap();

        assert_eq!(universe.get_attribute::<DummyAttribute>(handle).unwrap().0, 2);
    }

    #[derive(Debug)]
    struct DummyAttribute2;
    impl Attribute for DummyAttribute2 {}

    #[test]
    fn can_gather_entities() {
        let mut universe = Universe::default();
        let first_handle = universe.add_entity();
        universe
            .set_attribute(first_handle, DummyAttribute(2))
            .unwrap();

        let second_handle = universe.add_entity();
        universe
            .set_attribute(second_handle, DummyAttribute(1))
            .unwrap();

            universe.set_attribute(second_handle, DummyAttribute2).unwrap();

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

        assert_eq!(two.len(), 2);
        assert!(two.contains(&first_handle));
        assert!(two.contains(&second_handle));
    }
}
