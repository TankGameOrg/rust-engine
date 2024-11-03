use std::{any::TypeId, hash::Hash, marker::PhantomData};

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
pub trait AnyAttribute: AsAny + std::fmt::Debug {
    /// Get the name of this attribute
    fn get_name(&self) -> &'static str;

    /// Get the TypeId of this attribute's value
    fn get_value_type_id(&self) -> TypeId;
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
///
/// Each attribute has a name and value type.  For example we can
/// create an attribute called speed that stores a u32
/// ```
/// # use tank_game::rules::infrastructure::ecs::Attribute;
/// let speed = Attribute::<u32>::new("speed");
/// ```
/// or define an attribute that holds a struct
/// ```
/// # use tank_game::rules::infrastructure::ecs::{Attribute,AttributeValue};
/// #[derive(Debug)]
/// enum PetType {
///     Cat,
///     Dog,
/// }
///
/// #[derive(Debug)]
/// struct Pet {
///     pet: PetType,
///     name: &'static str,
/// }
///
/// impl AttributeValue for Pet {}
///
/// let pet = Attribute::<Pet>::new("pet");
/// ```
#[derive(Debug)]
pub struct Attribute<ValueType: AttributeValue> {
    name: &'static str,
    phantom: PhantomData<ValueType>,
}

impl<ValueType: AttributeValue> PartialEq for Attribute<ValueType> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<ValueType: AttributeValue> Attribute<ValueType> {
    /// Create a new attribute with the specified name
    #[inline]
    pub const fn new(name: &'static str) -> Attribute<ValueType> {
        Attribute {
            name,
            phantom: PhantomData,
        }
    }
}

impl<ValueType: AttributeValue> AnyAttribute for Attribute<ValueType> {
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn get_value_type_id(&self) -> TypeId {
        TypeId::of::<ValueType>()
    }
}

/// Shorthand to define a new attribute globally
///
/// We can define a new attribute DAMAGE_PER_TRUN like so
/// ```
/// # use tank_game::attribute;
/// attribute!(DAMAGE_PER_TURN: u32);
/// ```
#[macro_export]
macro_rules! attribute {
    ($access:vis $name:ident: $type:ty) => {
        pub static $name: $crate::rules::infrastructure::ecs::Attribute<$type> =
            $crate::rules::infrastructure::ecs::Attribute::new(stringify!($name));
    };
}

// A basic attribute for writing unit tests
#[cfg(test)]
attribute!(pub DUMMY_ATTRIBUTE: u32);
