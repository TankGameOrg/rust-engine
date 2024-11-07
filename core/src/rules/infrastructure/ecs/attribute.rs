use std::{any::TypeId, hash::Hash};

use as_any::AsAny;

/// The common ancestor for all attribute values
pub trait AttributeValue: AsAny + std::fmt::Debug + Send + Sync {}

impl AsRef<dyn AttributeValue> for dyn AttributeValue {
    fn as_ref(&self) -> &dyn AttributeValue {
        self
    }
}

/// Allow attributes to use u32
impl AttributeValue for u32 {}

/// AnyAttribute can be used to accept attributes of any type dynamically (basically `Attribute<impl Any>`)
pub trait AnyAttribute: AsAny {
    /// Get the name of this attribute
    fn get_name(&self) -> &'static str;

    /// Get the TypeId of this attribute's value
    fn get_value_type_id(&self) -> TypeId;
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
pub trait Attribute<ValueType: AttributeValue>: AnyAttribute {
    fn as_any_attribute(&self) -> &dyn AnyAttribute;
}

impl<ValueType: AttributeValue> std::fmt::Debug for dyn Attribute<ValueType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_any_attribute().fmt(f)
    }
}

impl<ValueType: AttributeValue> PartialEq for dyn Attribute<ValueType> {
    fn eq(&self, other: &Self) -> bool {
        self.get_name() == other.get_name()
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
/// # use tank_game_core::rules::infrastructure::ecs::{Attribute,AttributeValue};
/// #
/// #[derive(Debug)]
/// enum PetType {
///     Cat,
///     Dog,
/// }
///
/// #[derive(Debug)]
/// struct PetValue {
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
    ($access:vis $name:ident: $type:ty) => {
        $access struct $name;

        impl $crate::rules::infrastructure::ecs::AnyAttribute for $name {
            fn get_name(&self) -> &'static str {
                stringify!($name)
            }
        
            fn get_value_type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<$type>()
            }
        }

        impl $crate::rules::infrastructure::ecs::Attribute<$type> for $name {
            fn as_any_attribute(&self) -> &dyn $crate::rules::infrastructure::ecs::AnyAttribute {
                self
            }
        }
    };
}

// A basic attribute for writing unit tests
#[cfg(test)]
attribute!(pub DummyAttribute: u32);
