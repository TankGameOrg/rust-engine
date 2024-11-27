use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use super::Attribute;

/// The integer type used as a bit field in attribute values
type SignatureBitField = u64;

/// The ID type used for an attribute
pub type AttributeId = usize;

/// The highest attribute value allowed
///
/// This must be equal to the u* for the signature::SignatureBitField.  So if signature::SignatureBitField is a u64 MAX_ATTRIBUTE_ID should be 63
pub const MAX_NUM_ATTRIBUTES: AttributeId = 64;

/// A list of attributes that a given entity has
#[derive(Default)]
pub struct EntitySignature {
    bits: SignatureBitField,
}

impl EntitySignature {
    /// Check if the entity has a specific attribute
    pub fn has(&self, attribute_id: AttributeId) -> bool {
        self.bits & (1 << attribute_id) > 0
    }

    /// Add an attribute to the list for this entity
    pub fn add(&mut self, attribute_id: AttributeId) {
        self.bits |= 1 << attribute_id;
    }

    /// Remove an attribute from the list for this entity
    pub fn remove(&mut self, attribute_id: AttributeId) {
        self.bits &= !(1 << attribute_id);
    }

    /// Iterate the attributes owned by this entity
    pub(super) fn iter_attribute_ids(&self) -> AttributeIdIter {
        AttributeIdIter::new(self.bits)
    }
}

impl std::fmt::Debug for EntitySignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<EntitySignature {:#b}>", self.bits))
    }
}

pub(super) struct AttributeIdIter {
    expected_value: SignatureBitField,
    current_id: AttributeId,
}

/// Iterate the attribute IDs in a entity signature
impl AttributeIdIter {
    pub(super) fn new(expected_value: SignatureBitField) -> AttributeIdIter {
        AttributeIdIter {
            expected_value,
            current_id: 0,
        }
    }
}

impl Iterator for AttributeIdIter {
    type Item = AttributeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.expected_value == 0 {
            return None;
        }

        while self.expected_value & 1 != 1 {
            self.current_id += 1;
            self.expected_value >>= 1;
        }

        // Advance past the
        self.current_id += 1;
        self.expected_value >>= 1;

        Some(self.current_id - 1)
    }
}

#[derive(Debug, Default)]
pub(super) struct AttributeIdMap {
    mappings: HashMap<TypeId, AttributeId>,
}

impl AttributeIdMap {
    /// Assign an attribute ID to an attribute
    fn assign<T: Attribute>(&mut self) -> AttributeId {
        let id = self.mappings.len();
        assert!(id < MAX_NUM_ATTRIBUTES);
        self.mappings.insert(TypeId::of::<T>(), id);
        id
    }

    /// Get the attribute ID for an attribute
    #[inline]
    pub(super) fn get_id<T: Attribute>(&self) -> Option<AttributeId> {
        self.get_id_from_type_id(&TypeId::of::<T>())
    }

    /// Get the attribute ID for an attribute's TypeId
    #[inline]
    fn get_id_from_type_id(&self, type_id: &TypeId) -> Option<AttributeId> {
        self.mappings.get(type_id).cloned()
    }

    /// Get the attribute ID for this attribute or assign it one if one doesn't already exist
    pub(super) fn get_or_assign_id<T: Attribute>(&mut self) -> AttributeId {
        match self.get_id::<T>() {
            Some(attribute_id) => attribute_id,
            None => self.assign::<T>(),
        }
    }
}

pub struct CompiledSignature {
    mask: SignatureBitField,
    expected: SignatureBitField,
}

impl CompiledSignature {
    /// Check if this entity has all of the attributes required by the signature
    pub fn is_match(&self, entity_signature: &EntitySignature) -> bool {
        entity_signature.bits & self.mask == self.expected
    }
}

/// A list of attributes that should or shouldn't be present on a type of entity
#[derive(Clone, Debug)]
pub struct Signature {
    included: HashSet<TypeId>,
    excluded: HashSet<TypeId>,
}

impl Signature {
    /// Construct an empty signature that will match anything
    #[inline]
    pub fn empty() -> Signature {
        Signature {
            included: HashSet::new(),
            excluded: HashSet::new(),
        }
    }

    /// Construct a signature that expects an attribute to be present
    #[inline]
    pub fn with<T: Attribute>(mut self) -> Signature {
        self.included.insert(TypeId::of::<T>());
        self
    }

    /// Construct a signature that expects an attribute to not be present
    #[inline]
    pub fn without<T: Attribute>(mut self) -> Signature {
        self.excluded.insert(TypeId::of::<T>());
        self
    }

    /// Iterate the attribute ids that this signature expects to be included in this entity
    ///
    /// If an attribute in this signature hasn't been assigned an ID yet it will be skipped
    pub(super) fn iter_included<'iter>(
        &'iter self,
        id_map: &'iter AttributeIdMap,
    ) -> impl Iterator<Item = AttributeId> + 'iter {
        self.included
            .iter()
            .filter_map(|type_id| id_map.get_id_from_type_id(type_id))
    }

    /// Convert the signature into a format that can be matched against an entity signature
    ///
    /// If any of the attributes in this signature have not been assigned an ID yet the signature will not compile (return None)
    pub(super) fn compile(&self, mapping: &AttributeIdMap) -> Option<CompiledSignature> {
        let mut expected = 0;

        for included in &self.included {
            expected |= 1 << mapping.get_id_from_type_id(included)?;
        }

        let mut mask = expected;

        for excluded in &self.excluded {
            mask |= 1 << mapping.get_id_from_type_id(excluded)?;
        }

        Some(CompiledSignature { mask, expected })
    }
}

#[macro_export]
macro_rules! signature {
    () => {
        $crate::rules::infrastructure::ecs::Signature::empty()
    };

    ($attr:ty) => {
        signature!().with::<$attr>()
    };

    (!$attr:ty) => {
        signature!().without::<$attr>()
    };

    ($attr:ty, $($token:tt)+) => {
        signature!($($token)+).with::<$attr>()
    };

    (!$attr:ty, $($token:tt)+) => {
        signature!(!$($token)+).without::<$attr>()
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct Foo;
    #[derive(Debug)]
    struct Bar;
    #[derive(Debug)]
    struct Baz;

    impl Attribute for Foo {}
    impl Attribute for Bar {}
    impl Attribute for Baz {}

    const FOO_ID: AttributeId = 1;
    const BAR_ID: AttributeId = 2;
    const BAZ_ID: AttributeId = 3;

    #[test]
    fn entity_sigature_basics() {
        let mut entity_sig = EntitySignature::default();
        entity_sig.add(FOO_ID);
        entity_sig.add(BAR_ID);

        assert!(entity_sig.has(FOO_ID));
        assert!(!entity_sig.has(BAZ_ID));

        entity_sig.remove(FOO_ID);

        assert!(!entity_sig.has(FOO_ID));
        assert!(entity_sig.has(BAR_ID));
    }

    #[test]
    fn entity_sigature_iteration() {
        let mut entity_sig = EntitySignature::default();
        entity_sig.add(FOO_ID);
        entity_sig.add(BAZ_ID);

        let mut iter = entity_sig.iter_attribute_ids();
        assert_eq!(iter.next(), Some(FOO_ID));
        assert_eq!(iter.next(), Some(BAZ_ID));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn id_map_can_assign_ids_to_attributes() {
        let mut id_map = AttributeIdMap::default();
        id_map.assign::<Foo>();

        assert!(id_map.get_id::<Foo>().is_some());
        assert!(id_map.get_id::<Bar>().is_none());
        assert_ne!(
            id_map.get_or_assign_id::<Bar>(),
            id_map.get_id::<Foo>().unwrap()
        );
        assert_eq!(
            id_map.get_or_assign_id::<Bar>(),
            id_map.get_or_assign_id::<Bar>()
        );
    }

    #[test]
    fn signature_matches_entity() {
        let mut id_map = AttributeIdMap::default();

        let mut foo_baz_entity = EntitySignature::default();
        foo_baz_entity.add(id_map.get_or_assign_id::<Foo>());
        foo_baz_entity.add(id_map.get_or_assign_id::<Baz>());

        let mut bar_baz_entity = EntitySignature::default();
        bar_baz_entity.add(id_map.get_or_assign_id::<Bar>());
        bar_baz_entity.add(id_map.get_or_assign_id::<Baz>());

        let baz_sig = signature!(Baz).compile(&id_map).unwrap();
        assert!(baz_sig.is_match(&foo_baz_entity));
        assert!(baz_sig.is_match(&bar_baz_entity));

        let bar_sig = signature!(!Bar).compile(&id_map).unwrap();
        assert!(bar_sig.is_match(&foo_baz_entity));
        assert!(!bar_sig.is_match(&bar_baz_entity));

        let foo_baz_sig = signature!(Foo, Baz).compile(&id_map).unwrap();
        assert!(foo_baz_sig.is_match(&foo_baz_entity));
        assert!(!foo_baz_sig.is_match(&bar_baz_entity));
    }

    #[test]
    fn iterate_signature_attributes() {
        let mut id_map = AttributeIdMap::default();
        id_map.assign::<Foo>();
        id_map.assign::<Bar>();

        let foo_bar_signature = signature!(Foo, Bar);
        let ids: Vec<AttributeId> = foo_bar_signature.iter_included(&id_map).collect();
        assert!(ids.contains(&id_map.get_id::<Foo>().unwrap()));
        assert!(ids.contains(&id_map.get_id::<Bar>().unwrap()));
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn unassigned_attributes() {
        let mut id_map = AttributeIdMap::default();
        id_map.assign::<Foo>();

        let foo_baz_signature = signature!(Foo, Baz);
        let mut iter = foo_baz_signature.iter_included(&id_map);
        assert_eq!(iter.next(), Some(id_map.get_id::<Foo>().unwrap()));
        assert_eq!(iter.next(), None);

        assert!(foo_baz_signature.compile(&id_map).is_none());
    }
}
