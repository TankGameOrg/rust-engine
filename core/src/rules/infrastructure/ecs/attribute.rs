use std::{any::TypeId, hash::Hash, marker::PhantomData};

use as_any::AsAny;

/// The ID type used for an attribute
pub type AttributeId = usize;

/// The highest attribute value allowed
/// 
/// This must be 1 less than the u* for the signature::SignatureBitField.  So if signature::SignatureBitField is a u64 MAX_ATTRIBUTE_ID should be 63
pub const MAX_NUM_ATTRIBUTES: AttributeId = 63;

static mut NEXT_ATTRIBUTE_ID: AttributeId = 0;

/// Get a unique ID to use for this attribute
fn assign_attribute_id() -> AttributeId {
    unsafe {
        let id = NEXT_ATTRIBUTE_ID;
        assert!(id < MAX_NUM_ATTRIBUTES, "Too many attribute IDs have been assigned");
        NEXT_ATTRIBUTE_ID += 1;
        id
    }
}

/// The common ancestor for all attribute values
pub trait AttributeValue: AsAny + std::fmt::Debug + Send + Sync {}

// Allow attributes to use u32 and bool
impl AttributeValue for u32 {}
impl AttributeValue for bool {}

/// AnyAttribute can be used to accept attributes of any type dynamically (basically `Attribute<impl Any>`)
pub trait AnyAttribute: AsAny {
    /// Get the ID of this attribute
    fn get_attribute_id(&self) -> AttributeId;

    /// Get the name of this attribute
    fn get_name(&self) -> &'static str;

    /// Get the TypeId of this attribute's value
    fn get_value_type_id(&self) -> TypeId;

    /// Check if an attrbute is just a flag (has no value)
    fn is_flag(&self) -> bool;
}

impl std::fmt::Debug for dyn AnyAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<Attribute {}>", self.get_name()))
    }
}

impl PartialEq for dyn AnyAttribute {
    fn eq(&self, other: &Self) -> bool {
        self.get_name() == other.get_name() && self.get_value_type_id() == other.get_value_type_id()
    }
}

impl Eq for dyn AnyAttribute {}

impl Hash for dyn AnyAttribute {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_name().hash(state);
        self.get_value_type_id().hash(state);
    }
}

/// An attribute that can be used to access/store data on an entity
pub struct Attribute<ValueType: AttributeValue> {
    attribute_id: AttributeId,
    attribute_name: &'static str,
    phantom: PhantomData<ValueType>,
}

impl<T: AttributeValue> Attribute<T> {
    pub fn new(name: &'static str) -> Attribute<T> {
        Attribute {
            attribute_id: assign_attribute_id(),
            attribute_name: name,
            phantom: PhantomData,
        }
    }
}

impl<T: AttributeValue> AnyAttribute for Attribute<T> {
    fn get_attribute_id(&self) -> AttributeId {
        self.attribute_id
    }

    fn get_name(&self) -> &'static str {
        self.attribute_name
    }

    fn get_value_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn is_flag(&self) -> bool {
        false
    }
}

impl<ValueType: AttributeValue> std::fmt::Debug for Attribute<ValueType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn AnyAttribute).fmt(f)
    }
}

impl<ValueType: AttributeValue> PartialEq for Attribute<ValueType> {
    fn eq(&self, other: &Self) -> bool {
        self.get_attribute_id() == other.get_attribute_id()
    }
}

pub struct FlagAttribute {
    attribute_id: AttributeId,
    attribute_name: &'static str,
}

impl FlagAttribute {
    pub fn new(name: &'static str) -> FlagAttribute {
        FlagAttribute {
            attribute_id: assign_attribute_id(),
            attribute_name: name,
        }
    }
}
impl AnyAttribute for FlagAttribute {
    fn get_attribute_id(&self) -> AttributeId {
        self.attribute_id
    }

    fn get_name(&self) -> &'static str {
        self.attribute_name
    }

    fn get_value_type_id(&self) -> TypeId {
        TypeId::of::<()>()
    }

    fn is_flag(&self) -> bool {
        true
    }
}

impl std::fmt::Debug for FlagAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn AnyAttribute).fmt(f)
    }
}

impl PartialEq for FlagAttribute {
    fn eq(&self, other: &Self) -> bool {
        self.get_attribute_id() == other.get_attribute_id()
    }
}

/// Shorthand to define a new attribute globally
///
/// We can define a new attribute DamagePerTurn like so
/// ```
/// # use tank_game_core::attribute;
/// attribute!(DamagePerTurn: u32);
/// ```
///
/// or define an attribute that holds a struct
/// ```
/// # use tank_game_core::attribute;
/// # use tank_game_core::rules::infrastructure::ecs::{Attribute, AttributeValue};
/// #
/// #[derive(Debug)]
/// pub enum PetType {
///     Cat,
///     Dog,
/// }
///
/// #[derive(Debug)]
/// pub struct PetValue {
///     pet: PetType,
///     name: &'static str,
/// }
///
/// impl AttributeValue for PetValue {}
///
/// attribute!(Pet: PetValue);
/// ```
#[macro_export]
macro_rules! attribute {
    ($name:ident: $type:ty) => {
        lazy_static::lazy_static! {
            pub static ref $name: $crate::rules::infrastructure::ecs::Attribute<$type> = $crate::rules::infrastructure::ecs::Attribute::new(stringify!($name));
        }
    };

    (flag $name:ident) => {
        lazy_static::lazy_static! {
            pub static ref $name: $crate::rules::infrastructure::ecs::FlagAttribute = $crate::rules::infrastructure::ecs::FlagAttribute::new(stringify!($name));
        }
    };
}

// A basic attribute for writing unit tests
#[cfg(test)]
attribute!(DummyAttribute: u32);
