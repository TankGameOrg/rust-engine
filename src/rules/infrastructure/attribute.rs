use std::marker::PhantomData;

use as_any::AsAny;

/// The common ancestor for all attribute values
pub trait AttributeValue: AsAny + std::fmt::Debug {}

impl AsRef<dyn AttributeValue> for dyn AttributeValue {
    fn as_ref(&self) -> &dyn AttributeValue {
        self
    }
}

/// Allow attributes to use u32
impl AttributeValue for u32 {}

/// An attribute that can be used to access/store data on an entity
///
/// Each attribute has a name and value type.  For example we can
/// create an attribute called speed that stores a u32
/// ```
/// # use tank_game::rules::infrastructure::attribute::Attribute;
/// let speed = Attribute::<u32>::new("speed");
/// ```
/// or define an attribute that holds a struct
/// ```
/// # use tank_game::rules::infrastructure::attribute::{Attribute,AttributeValue};
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
#[derive(Hash, Eq, Debug)]
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

    /// Get the name of this attribute
    #[inline]
    pub fn get_name(&self) -> &'static str {
        self.name
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
    ($name:ident: $type:ty) => {
        pub static $name: $crate::rules::infrastructure::attribute::Attribute<$type> =
            $crate::rules::infrastructure::attribute::Attribute::new(stringify!($name));
    };
}

// A basic attribute for writing unit tests
#[cfg(test)]
attribute!(DUMMY_ATTRIBUTE: u32);
