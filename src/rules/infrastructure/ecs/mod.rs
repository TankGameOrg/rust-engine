//! The entity component system used to represent all of the objects in the game 

mod attribute;
mod entity;
mod universe;
mod transaction;

pub use attribute::{Attribute, AnyAttribute, AttributeValue};
pub use entity::Entity;
pub use universe::{GatheredResult, Handle, Index, Universe};
pub use transaction::*;
