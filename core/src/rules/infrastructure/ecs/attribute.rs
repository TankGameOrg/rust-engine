use as_any::AsAny;

/// The common ancestor for all attribute values
pub trait Attribute: std::fmt::Debug + Send + Sync + AsAny {}

// A basic attribute for writing unit tests
#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub struct DummyAttribute(pub u32);

#[cfg(test)]
impl Attribute for DummyAttribute {}
