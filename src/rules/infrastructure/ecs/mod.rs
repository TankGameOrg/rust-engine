//! The entity component system used to represent all of the objects in the game 

mod attribute;
mod entity;
mod pool;
mod transaction;

pub use attribute::{Attribute, AnyAttribute, AttributeValue};
pub use entity::Entity;
pub use pool::{GatheredResult, Handle, Index, Pool};
pub use transaction::*;
