use super::{attribute::AttributeId, AnyAttribute};

/// The integer type used as a bit field in attribute values
type SignatureBitField = u64;

/// A list of attributes that a given entity has
#[derive(Debug)]
pub struct EntitySignature {
    bits: SignatureBitField,
}

impl Default for EntitySignature {
    fn default() -> Self {
        EntitySignature {
            bits: 0,
        }
    }
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


/// A list of attributes that should or shouldn't be present on a type of entity
#[derive(Debug, Clone, Copy)]
pub struct Signature {
    mask: SignatureBitField,
    expected_value: SignatureBitField,
}

impl Signature {
    /// Construct an empty signature that will match anything
    #[inline]
    pub fn empty() -> Signature {
        Signature {
            mask: 0,
            expected_value: 0,
        }
    }

    /// Construct a signature that expects an attribute to be present
    #[inline]
    pub fn with<T: AnyAttribute>(attribute: &T) -> Signature {
        let bit = 1 << attribute.get_attribute_id();

        Signature {
            mask: bit,
            expected_value: bit,
        }
    }

    /// Construct a signature that expects an attribute to not be present
    #[inline]
    pub fn without(attribute: &dyn AnyAttribute) -> Signature {
        let bit = 1 << attribute.get_attribute_id();

        Signature {
            mask: bit,
            expected_value: 0,
        }
    }

    /// Combine the requirements of two signatures
    #[inline]
    pub fn combine(self, other: Signature) -> Signature {
        Signature {
            mask: self.mask | other.mask,
            expected_value: self.expected_value | other.expected_value,
        }
    }

    #[inline]
    pub(super) fn is_match(&self, signature: &EntitySignature) -> bool {
        signature.bits & self.mask == self.expected_value
    }

    pub(super) fn iter_attribute_ids(&self) -> AttributeIdIter {
        AttributeIdIter::new(self.expected_value)
    }
}

pub(super) struct AttributeIdIter {
    expected_value: SignatureBitField,
    current_id: AttributeId,
}

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
            return None
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

#[macro_export]
macro_rules! signature {
    () => {
        $crate::rules::infrastructure::ecs::Signature::empty()
    };

    ($attr:ident) => {
        $crate::rules::infrastructure::ecs::Signature::with(&*$attr)
    };

    (!$attr:ident) => {
        $crate::rules::infrastructure::ecs::Signature::without(&*$attr)
    };

    ($attr:ident, $($token:tt)+) => {
        signature!($attr).combine(signature!($($token)+))
    };

    (!$attr:ident, $($token:tt)+) => {
        signature!(!$attr).combine(signature!($($token)+))
    };
}

#[cfg(test)]
mod test {
    use crate::attribute;

    use super::*;

    attribute!(FOO: u32);
    attribute!(BAR: u32);
    attribute!(BAZ: u32);

    #[test]
    fn entity_sigature_basics() {
        let mut entity_sig = EntitySignature::default();
        entity_sig.add(FOO.get_attribute_id());
        entity_sig.add(BAR.get_attribute_id());

        assert!(entity_sig.has(FOO.get_attribute_id()));
        assert!(!entity_sig.has(BAZ.get_attribute_id()));

        entity_sig.remove(FOO.get_attribute_id());
        
        assert!(!entity_sig.has(FOO.get_attribute_id()));
        assert!(entity_sig.has(BAR.get_attribute_id()));
    }

    #[test]
    fn entity_sigature_iteration() {
        let mut entity_sig = EntitySignature::default();
        entity_sig.add(FOO.get_attribute_id());
        entity_sig.add(BAZ.get_attribute_id());

        let mut iter = entity_sig.iter_attribute_ids();
        assert_eq!(iter.next(), Some(FOO.get_attribute_id()));
        assert_eq!(iter.next(), Some(BAZ.get_attribute_id()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn signature_matches_entity() {
        let mut foo_baz_entity = EntitySignature::default();
        foo_baz_entity.add(FOO.get_attribute_id());
        foo_baz_entity.add(BAZ.get_attribute_id());

        let mut bar_baz_entity = EntitySignature::default();
        bar_baz_entity.add(BAR.get_attribute_id());
        bar_baz_entity.add(BAZ.get_attribute_id());

        let baz_sig = signature!(BAZ);
        assert!(baz_sig.is_match(&foo_baz_entity));
        assert!(baz_sig.is_match(&bar_baz_entity));

        let bar_sig = signature!(!BAR);
        assert!(bar_sig.is_match(&foo_baz_entity));
        assert!(!bar_sig.is_match(&bar_baz_entity));

        let foo_baz_sig = signature!(FOO, BAZ);
        assert!(foo_baz_sig.is_match(&foo_baz_entity));
        assert!(!foo_baz_sig.is_match(&bar_baz_entity));
    }

    #[test]
    fn iterate_signature_attributes() {
        let foo_bar_signature = signature!(FOO, BAR);
        let mut iter = foo_bar_signature.iter_attribute_ids();
        assert_eq!(iter.next(), Some(FOO.get_attribute_id()));
        assert_eq!(iter.next(), Some(BAR.get_attribute_id()));
        assert_eq!(iter.next(), None);
    }
}