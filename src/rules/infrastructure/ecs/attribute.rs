use std::marker::PhantomData;

/// An attribute that can be used to access/store data on an entity
///
/// Each attribute has a name and value type.  For example we can
/// create an attribute called speed that stores a u32
/// ```
/// let speed = Attribute<u32>::new("speed");
/// ```
#[derive(Hash, Eq, Debug)]
pub struct Attribute<ValueType> {
    name: &'static str,
    phantom: PhantomData<ValueType>,
}

impl<ValueType> PartialEq for Attribute<ValueType> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<ValueType> Attribute<ValueType> {
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
/// attribute!(DAMAGE_PER_TURN: u8);
/// ```
#[macro_export]
macro_rules! attribute {
    ($name:ident: $type:ty) => {
        static $name: $crate::rules::infrastructure::ecs::attribute::Attribute<$type> =
            $crate::rules::infrastructure::ecs::attribute::Attribute::new(stringify!($name));
    };
}
