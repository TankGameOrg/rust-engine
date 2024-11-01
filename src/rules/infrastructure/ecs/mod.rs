//! The entity component system used to represent all of the objects in the game 

mod attribute;
mod container;
mod pool;
mod transaction;

pub use attribute::{Attribute, AttributeValue};
pub use container::AttributeContainer;
pub use pool::{GatheredResult, Handle, Index, Pool};
pub use transaction::*;
