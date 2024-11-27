use std::{
    any::{self, type_name, Any},
    collections::HashMap,
    error::Error,
};

use as_any::{AsAny, Downcast};

use crate::basic_error;

use super::{
    attribute::Attribute,
    signature::{AttributeId, AttributeIdIter, AttributeIdMap, EntitySignature, Signature},
    store::{AttributeStore, DefaultAttributeStore, Handle, HandleIterator},
    Query,
};

/// The internal, object safe interface for storing attributes without knowing the underlying implementation
///
/// This trait exists to allow the Universe to store a map of AttributeStores for a variety of attribute types
/// without knowing the type of the underlying attribute store.  While allowing exposing a compile check type
/// safe interface to our clients.
trait GenericAttributeStore: std::fmt::Debug + AsAny {
    /// See [`AttributeStore::len`]
    fn len(&self) -> usize;

    /// See [`AttributeStore::iter_handles`]
    fn iter_handles(&self) -> HandleIterator;

    /// See [`AttributeStore::remove_attribute`]
    fn remove_attribute(&mut self, handle: Handle);

    /// See [`AttributeStore::get_attribute`]
    fn get_attribute(&self, handle: Handle) -> &dyn Attribute;

    /// See [`AttributeStore::set_attribute`]
    fn set_attribute(&mut self, handle: Handle, value: Box<dyn Any>) -> Result<(), Box<dyn Error>>;
}

impl<T: AttributeStore> GenericAttributeStore for T {
    fn len(&self) -> usize {
        self.len()
    }

    fn iter_handles(&self) -> HandleIterator {
        self.iter_handles()
    }

    fn remove_attribute(&mut self, handle: Handle) {
        self.remove_attribute(handle);
    }

    fn get_attribute(&self, handle: Handle) -> &dyn Attribute {
        self.get_attribute(handle)
    }

    fn set_attribute(&mut self, handle: Handle, value: Box<dyn Any>) -> Result<(), Box<dyn Error>> {
        let value_name = value.as_ref().type_id();

        match value.downcast::<T::StoredAttribute>() {
            Err(_) => Err(basic_error!(
                "Expected attribute of type {} but got {:?}",
                type_name::<T>(),
                value_name
            )),
            Ok(value) => self.set_attribute(handle, *value),
        }
    }
}

/// A collection of entities where each entity is made up one or more Attributes
#[derive(Debug, Default)]
pub struct Universe {
    id_map: AttributeIdMap,
    entities: HashMap<Handle, EntitySignature>,
    stores: HashMap<AttributeId, Box<dyn GenericAttributeStore>>,
}

impl Universe {
    /// Add an entity to the universe
    /// ```
    /// # use tank_game_core::rules::infrastructure::ecs::{Universe, Attribute};
    /// let mut universe = Universe::default();
    /// #
    /// # #[derive(Debug)]
    /// # struct DummyAttribute;
    /// # impl Attribute for DummyAttribute {}
    ///
    /// // Get a Handle to the new entity
    /// let handle = universe.add_entity()
    ///     .set(DummyAttribute)
    ///     .as_handle()?;
    ///
    /// // Get a reference to the new entity
    /// let entity = universe.add_entity().as_entity()?;
    ///
    /// // Get a mutable reference to the new entity
    /// let mut entity = universe.add_entity()
    ///     .set(DummyAttribute)
    ///     .as_entity_mut()?;
    /// #
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_entity(&mut self) -> EntityBuilder {
        let handle = Handle::new();
        self.entities.insert(handle, EntitySignature::default());

        EntityBuilder::new(self, handle)
    }

    fn get_signature(&self, handle: &Handle) -> Result<&EntitySignature, Box<dyn Error>> {
        self.entities
            .get(&handle)
            .ok_or(basic_error!("There is no entity for handle {:?}", handle))
    }

    fn get_signature_mut(
        &mut self,
        handle: &Handle,
    ) -> Result<&mut EntitySignature, Box<dyn Error>> {
        self.entities
            .get_mut(&handle)
            .ok_or(basic_error!("There is no entity for handle {:?}", handle))
    }

    /// Get an attribute's value from an entity
    ///
    /// If the entity doesn't exist or it doesn't exist we return an error
    fn get_attribute<T: Attribute>(&self, handle: Handle) -> Result<&T, Box<dyn Error>> {
        let attribute_id = self.id_map.get_id::<T>();
        let signature = self.get_signature(&handle)?;
        if attribute_id.is_none() || !signature.has(attribute_id.unwrap()) {
            return Err(basic_error!(
                "Entity {:?} does not have the attribute {:?} (signature = {:?})",
                handle,
                attribute_id,
                signature
            ));
        }

        let attribute_id = attribute_id.unwrap();
        let store = self.stores.get(&attribute_id).unwrap();

        store
            .get_attribute(handle)
            .downcast_ref()
            .ok_or(basic_error!(
                "Got the wrong type when reading {} from {:?}",
                any::type_name::<T>(),
                handle
            ))
    }

    /// Set an attribute's value for an entity
    ///
    /// If the entity doesn't exist we return an error
    fn set_attribute<T: Attribute>(
        &mut self,
        handle: Handle,
        value: T,
    ) -> Result<(), Box<dyn Error>> {
        let attribute_id = self.id_map.get_or_assign_id::<T>();
        self.get_signature_mut(&handle)?.add(attribute_id);

        if !self.stores.contains_key(&attribute_id) {
            self.stores.insert(
                attribute_id,
                Box::new(DefaultAttributeStore::<T>::default()),
            );
        }

        let store = self.stores.get_mut(&attribute_id).unwrap();
        store.set_attribute(handle, Box::new(value))
    }

    /// Remove an attribute from an entity
    ///
    /// If the entity doesn't exist we return an error
    fn remove_attribute<T: Attribute>(&mut self, handle: Handle) -> Result<(), Box<dyn Error>> {
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
    fn has_attribute<T: Attribute>(&self, handle: Handle) -> Result<bool, Box<dyn Error>> {
        match self.id_map.get_id::<T>() {
            Some(attribute_id) => Ok(self.get_signature(&handle)?.has(attribute_id)),
            None => Ok(false),
        }
    }

    /// Iterate the attributes on an entity
    fn iter_attributes(&self, handle: Handle) -> Result<AttributeIter, Box<dyn Error>> {
        let signature = self.get_signature(&handle)?;

        Ok(AttributeIter {
            universe: self,
            attribute_id_iter: signature.iter_attribute_ids(),
            handle: handle,
        })
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

    /// Check if the specified entity matches the signature given
    #[inline]
    fn is_match(&self, handle: Handle, signature: Signature) -> bool {
        match signature.compile(&self.id_map) {
            Some(compiled) => compiled.is_match(self.get_signature(&handle).unwrap()),
            None => false,
        }
    }

    /// Get a reference to an entity
    #[inline]
    pub fn get_entity(&self, handle: Handle) -> Result<EntityRef, Box<dyn Error>> {
        // Verify that the entity exists
        self.get_signature(&handle)?;

        Ok(EntityRef {
            universe: self,
            handle,
        })
    }

    /// Get a mutable reference to an entity
    #[inline]
    pub fn get_entity_mut(&mut self, handle: Handle) -> Result<EntityMut, Box<dyn Error>> {
        // Verify that the entity exists
        self.get_signature(&handle)?;

        Ok(EntityMut {
            universe: self,
            handle,
        })
    }

    /// Collect the handles for all entities in the universe that match the specified signature
    pub fn gather_handles<'iter>(
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
        handle_iter.filter(move |handle| match &compiled_signature {
            Some(compiled) => compiled.is_match(self.entities.get(handle).unwrap()),
            None => false,
        })
    }

    /// Collect the all of the entities in the universe that match the specified signature
    #[inline]
    pub fn gather<'iter>(
        &'iter self,
        signature: Signature,
    ) -> impl Iterator<Item = EntityRef> + 'iter {
        self.gather_handles(signature).map(|handle| EntityRef {
            universe: self,
            handle,
        })
    }

    /// Set the structure used to store a specific type of attribute
    ///
    /// The store must be set before any attributes of its StoredAttribute type have been added to the Universe
    #[inline]
    pub fn set_attribute_store<S: AttributeStore>(&mut self, store: S) {
        let attribute_id = self.id_map.get_or_assign_id::<S::StoredAttribute>();
        assert_eq!(store.len(), 0);
        assert!(!self.stores.contains_key(&attribute_id));
        self.stores.insert(attribute_id, Box::new(store));
    }

    /// Preform an optimized lookup for a specific attribute
    #[inline]
    pub fn query_handle<'iter, Q: Query>(
        &'iter self,
        query: &'iter Q,
    ) -> Result<impl Iterator<Item = Handle> + 'iter, Box<dyn Error>> {
        let attribute_id = self.id_map.get_id::<Q::StoredAttribute>();
        if attribute_id.is_none() {
            return Ok(HandleIterator::empty());
        }

        let attribute_id = attribute_id.unwrap();
        let store = self.stores.get(&attribute_id).unwrap();
        let store: &Q::Store = store.as_ref()
            .downcast_ref()
            .ok_or(basic_error!("Query expects the store to be {} but it was {}.  Did you forget to call set_attribute_store?",
                type_name::<Q::Store>(), store.as_ref().type_name()))?;

        Ok(query.query(store))
    }

    /// Preform an optimized lookup for a specific attribute
    #[inline]
    pub fn query<'iter, Q: Query>(
        &'iter self,
        query: &'iter Q,
    ) -> Result<impl Iterator<Item = EntityRef> + 'iter, Box<dyn Error>> {
        Ok(self.query_handle(query)?.map(|handle| EntityRef {
            universe: self,
            handle,
        }))
    }
}

/// An iterator for the attributes in a entity
pub struct AttributeIter<'universe> {
    universe: &'universe Universe,
    attribute_id_iter: AttributeIdIter,
    handle: Handle,
}

impl<'universe> Iterator for AttributeIter<'universe> {
    type Item = &'universe dyn Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        let attribute_id = self.attribute_id_iter.next()?;
        let store = self.universe.stores.get(&attribute_id).unwrap();
        Some(store.get_attribute(self.handle))
    }
}

/// A helper for building entities
///
/// If any step of the build fails the entity will be removed
#[must_use]
pub struct EntityBuilder<'universe> {
    universe: &'universe mut Universe,
    handle: Handle,
    result: Result<(), Box<dyn Error>>,
}

impl<'universe> EntityBuilder<'universe> {
    fn new(universe: &'universe mut Universe, handle: Handle) -> EntityBuilder<'universe> {
        EntityBuilder {
            universe,
            handle,
            result: Ok(()),
        }
    }

    /// Set an attribute on the entity being build
    #[inline]
    pub fn set<T: Attribute>(mut self, value: T) -> Self {
        if self.result.is_ok() {
            if let Err(err) = self.universe.set_attribute(self.handle, value) {
                // Something went wrong try to remove the entity and mark this builder as failed
                self.universe.remove_entity(self.handle).unwrap();
                self.result = Err(err);
            }
        }

        self
    }

    /// Convert to a result indicating whether the entity was created
    #[inline]
    pub fn as_result(self) -> Result<(), Box<dyn Error>> {
        self.result
    }

    /// Short hand for `.as_result().unwrap()`
    #[inline]
    pub fn unwrap(self) {
        self.as_result().unwrap()
    }

    /// Get the handle for the newly created entity
    #[inline]
    pub fn as_handle(self) -> Result<Handle, Box<dyn Error>> {
        let handle = self.handle;
        self.result.map(|()| handle)
    }

    /// Get a read only reference to the entity
    #[inline]
    pub fn as_entity(self) -> Result<EntityRef<'universe>, Box<dyn Error>> {
        let entity = EntityRef {
            universe: self.universe,
            handle: self.handle,
        };

        self.result.map(move |()| entity)
    }

    /// Get a read write reference to the entity
    #[inline]
    pub fn as_entity_mut(self) -> Result<EntityMut<'universe>, Box<dyn Error>> {
        let entity_mut = EntityMut {
            universe: self.universe,
            handle: self.handle,
        };

        self.result.map(move |()| entity_mut)
    }
}

/// A reference to an immutable entity in a universe
#[must_use]
pub struct EntityRef<'universe> {
    universe: &'universe Universe,
    handle: Handle,
}

impl<'universe> EntityRef<'universe> {
    /// Get the handle for this entity
    #[inline]
    pub fn get_handle(&self) -> Handle {
        self.handle
    }

    /// Get an attribute's value from this entity
    /// ```
    /// # use tank_game_core::rules::infrastructure::ecs::{Universe, Attribute};
    /// let mut universe = Universe::default();
    /// #
    /// # #[derive(Debug)]
    /// # struct DummyAttribute(u32);
    /// # impl Attribute for DummyAttribute {}
    /// #
    /// # let handle = universe.add_entity()
    /// #    .set(DummyAttribute(1))
    /// #    .as_handle()?;
    /// let entity = universe.get_entity(handle)?;
    ///
    /// let DummyAttribute(_value) = entity.get()?;
    /// #
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn get<T: Attribute>(&self) -> Result<&'universe T, Box<dyn Error>> {
        self.universe.get_attribute(self.handle)
    }

    /// Check if this entity has an attribute
    /// ```
    /// # use tank_game_core::rules::infrastructure::ecs::{Universe, Attribute};
    /// let mut universe = Universe::default();
    /// #
    /// # #[derive(Debug)]
    /// # struct DummyAttribute(u32);
    /// # impl Attribute for DummyAttribute {}
    /// #
    /// # let handle = universe.add_entity()
    /// #    .set(DummyAttribute(1))
    /// #    .as_handle()?;
    /// let entity = universe.get_entity(handle)?;
    ///
    /// assert!(entity.has::<DummyAttribute>());
    /// #
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn has<T: Attribute>(&self) -> bool {
        self.universe.has_attribute::<T>(self.handle).unwrap()
    }

    /// Iterate the attributes on this entity
    /// ```
    /// # use tank_game_core::rules::infrastructure::ecs::{Universe, Attribute};
    /// let mut universe = Universe::default();
    /// #
    /// # #[derive(Debug)]
    /// # struct DummyAttribute(u32);
    /// # impl Attribute for DummyAttribute {}
    /// #
    /// # let handle = universe.add_entity()
    /// #    .set(DummyAttribute(1))
    /// #    .as_handle()?;
    /// let entity = universe.get_entity(handle)?;
    ///
    /// for attribute in &entity {
    ///     println!("Attribute: {:?}", attribute);
    /// }
    /// #
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn iter(&self) -> AttributeIter<'universe> {
        self.universe.iter_attributes(self.handle).unwrap()
    }

    /// Check if the this entity matches the signature given
    /// ```
    /// # use tank_game_core::signature;
    /// # use tank_game_core::rules::infrastructure::ecs::{Universe, Attribute};
    /// let mut universe = Universe::default();
    /// #
    /// # #[derive(Debug)]
    /// # struct DummyAttribute(u32);
    /// # impl Attribute for DummyAttribute {}
    /// #
    /// # let handle = universe.add_entity()
    /// #    .set(DummyAttribute(1))
    /// #    .as_handle()?;
    /// let entity = universe.get_entity(handle)?;
    ///
    /// assert!(entity.is_match(signature!(DummyAttribute)));
    /// #
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn is_match(&self, signature: Signature) -> bool {
        self.universe.is_match(self.handle, signature)
    }
}

impl<'universe> IntoIterator for &EntityRef<'universe> {
    type Item = &'universe dyn Attribute;
    type IntoIter = AttributeIter<'universe>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A reference to a mutable entity in a universe
#[must_use]
pub struct EntityMut<'universe> {
    universe: &'universe mut Universe,
    handle: Handle,
}

impl<'universe> EntityMut<'universe> {
    /// Get the handle for this entity
    #[inline]
    pub fn get_handle(&self) -> Handle {
        self.handle
    }

    /// Get an attribute's value from this entity
    ///
    /// Same as [`EntityRef::get`]
    #[inline]
    pub fn get<T: Attribute>(&self) -> Result<&T, Box<dyn Error>> {
        self.universe.get_attribute(self.handle)
    }

    /// Set an attribute's value for this entity
    /// ```
    /// # use tank_game_core::rules::infrastructure::ecs::{Universe, Attribute};
    /// let mut universe = Universe::default();
    /// #
    /// # #[derive(Debug)]
    /// # struct DummyAttribute(u32);
    /// # impl Attribute for DummyAttribute {}
    /// #
    /// # let handle = universe.add_entity().as_handle()?;
    /// let mut entity = universe.get_entity_mut(handle)?;
    ///
    /// entity.set(DummyAttribute(3));
    /// #
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn set<T: Attribute>(&mut self, value: T) -> Result<(), Box<dyn Error>> {
        self.universe.set_attribute(self.handle, value)
    }

    /// Remove an attribute from this entity
    /// ```
    /// # use tank_game_core::rules::infrastructure::ecs::{Universe, Attribute};
    /// let mut universe = Universe::default();
    /// #
    /// # #[derive(Debug)]
    /// # struct DummyAttribute(u32);
    /// # impl Attribute for DummyAttribute {}
    /// #
    /// # let handle = universe.add_entity().as_handle()?;
    /// let mut entity = universe.get_entity_mut(handle)?;
    ///
    /// entity.remove::<DummyAttribute>();
    /// #
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn remove<T: Attribute>(&mut self) {
        self.universe.remove_attribute::<T>(self.handle).unwrap();
    }

    /// Check if this entity has an attribute
    ///
    /// Same as [`EntityRef::has`]
    #[inline]
    pub fn has<T: Attribute>(&self) -> bool {
        self.universe.has_attribute::<T>(self.handle).unwrap()
    }

    /// Iterate the attributes on this entity
    ///
    /// Same as [`EntityRef::iter`]
    #[inline]
    pub fn iter<'iter>(&'iter self) -> AttributeIter<'iter>
    where
        'universe: 'iter,
    {
        self.universe.iter_attributes(self.handle).unwrap()
    }

    /// Check if the this entity matches the signature given
    ///
    /// Same as [`EntityRef::is_match`]
    #[inline]
    pub fn is_match(&self, signature: Signature) -> bool {
        self.universe.is_match(self.handle, signature)
    }
}

impl<'universe, 'iter: 'universe> IntoIterator for &'iter EntityMut<'universe> {
    type Item = &'universe dyn Attribute;
    type IntoIter = AttributeIter<'universe>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use any::TypeId;

    use crate::{rules::infrastructure::ecs::attribute::DummyAttribute, signature};

    use super::*;

    #[test]
    fn can_modify_and_retrieve_entities() {
        let mut universe = Universe::default();
        let handle = universe
            .add_entity()
            .set(DummyAttribute(2))
            .as_handle()
            .unwrap();

        let entity = universe.get_entity(handle).unwrap();
        assert_eq!(*entity.get::<DummyAttribute>().unwrap(), DummyAttribute(2));
    }

    #[test]
    fn can_remove_attributes() {
        let mut universe = Universe::default();
        let mut entity = universe.add_entity().as_entity_mut().unwrap();
        entity.set(DummyAttribute(2)).unwrap();
        assert!(entity.has::<DummyAttribute>());

        entity.remove::<DummyAttribute>();
        assert!(!entity.has::<DummyAttribute>());
        entity.remove::<DummyAttribute>();
        assert!(!entity.has::<DummyAttribute>());
    }

    #[derive(Debug)]
    struct DummyAttribute2;
    impl Attribute for DummyAttribute2 {}

    fn make_gather_universe() -> (Universe, Handle, Handle) {
        let mut universe = Universe::default();
        let first_handle = universe
            .add_entity()
            .set(DummyAttribute(2))
            .as_handle()
            .unwrap();

        let second_handle = universe
            .add_entity()
            .set(DummyAttribute(1))
            .set(DummyAttribute2)
            .as_handle()
            .unwrap();

        let _ = universe.add_entity();

        (universe, first_handle, second_handle)
    }

    #[test]
    fn can_gather_handles() {
        let (universe, first_handle, second_handle) = make_gather_universe();

        // Gather one of the entities
        let one: Vec<Handle> = universe
            .gather_handles(signature!(DummyAttribute2))
            .collect();

        assert_eq!(one.len(), 1);
        assert_eq!(one[0], second_handle);

        // Gather both of the ones with attributes
        let two: Vec<Handle> = universe
            .gather_handles(signature!(DummyAttribute))
            .collect();

        assert_eq!(two.len(), 2);
        assert!(two.contains(&first_handle));
        assert!(two.contains(&second_handle));
    }

    #[test]
    fn can_gather_entities() {
        let (universe, _, _) = make_gather_universe();

        let dummy_total = universe
            .gather(signature!(DummyAttribute))
            .map(|entity| entity.get::<DummyAttribute>().unwrap().0)
            .reduce(|a, b| a + b);

        assert_eq!(dummy_total.unwrap(), 3);
    }

    #[test]
    fn can_iterate_attributes() {
        let mut universe = Universe::default();
        let entity = universe
            .add_entity()
            .set(DummyAttribute(1))
            .set(DummyAttribute2)
            .as_entity()
            .unwrap();

        let attributes: HashSet<TypeId> = entity
            .into_iter()
            .map(|attr| attr.as_any().type_id())
            .collect();

        assert!(attributes.contains(&TypeId::of::<DummyAttribute>()));
        assert!(attributes.contains(&TypeId::of::<DummyAttribute2>()));
        assert_eq!(attributes.len(), 2);
    }

    #[derive(Debug, Default)]
    struct TestStore {
        handle_to_value: HashMap<Handle, DummyAttribute>,
    }

    impl AttributeStore for TestStore {
        type StoredAttribute = DummyAttribute;

        fn get_attribute(&self, handle: Handle) -> &Self::StoredAttribute {
            self.handle_to_value.get(&handle).unwrap()
        }

        fn set_attribute(
            &mut self,
            handle: Handle,
            value: Self::StoredAttribute,
        ) -> Result<(), Box<dyn Error>> {
            if value.0 > 10 {
                Err(basic_error!("Value must not be more than 10"))
            } else {
                self.handle_to_value.insert(handle, value);
                Ok(())
            }
        }

        fn iter_handles(&self) -> HandleIterator {
            HandleIterator::empty()
        }

        fn len(&self) -> usize {
            self.handle_to_value.len()
        }

        fn remove_attribute(&mut self, handle: Handle) {
            self.handle_to_value.remove(&handle);
        }
    }

    struct LessThan(u32);

    impl Query for LessThan {
        type StoredAttribute = DummyAttribute;
        type Store = TestStore;

        fn query<'iter>(&'iter self, store: &'iter Self::Store) -> HandleIterator<'iter> {
            HandleIterator::new(
                store
                    .handle_to_value
                    .iter()
                    .filter(|(_, value)| value.0 < self.0)
                    .map(|(handle, _)| handle),
            )
        }
    }

    #[test]
    fn custom_store_test() {
        let mut universe = Universe::default();
        universe.set_attribute_store(TestStore::default());

        universe.add_entity().set(DummyAttribute(3)).unwrap();
        universe.add_entity().set(DummyAttribute(7)).unwrap();

        let matches: Vec<&DummyAttribute> = universe
            .query(&LessThan(5))
            .unwrap()
            .map(|entity| entity.get().unwrap())
            .collect();

        assert_eq!(matches, vec![&DummyAttribute(3)]);
    }

    #[test]
    fn custom_store_rejects_attributes() {
        let mut universe = Universe::default();
        universe.set_attribute_store(TestStore::default());

        let result = universe.add_entity().set(DummyAttribute(11)).as_result();
        assert!(result.is_err());
    }
}
