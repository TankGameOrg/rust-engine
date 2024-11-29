//! The entity component system used to represent all of the objects in the game
//!
//! Here's an example of how to create a universe and add some basic entities to it
//! ```
//! # use std::error::Error;
//! # use std::vec::Vec;
//! # use tank_game_core::*;
//! # use tank_game_core::rules::infrastructure::ecs::*;
//! #
//! #[derive(Debug)]
//! struct Health(u32);
//! impl Attribute for Health {}
//!
//! fn damage_living(universe: &mut Universe) -> Result<(), Box<dyn Error>> {
//!     let living: Vec<(Handle, u32)> = universe.find_signature(signature!(Health))
//!         .filter(|entity| entity.get::<Health>().unwrap().0 > 0)
//!         .map(|entity| (entity.get_handle(), entity.get::<Health>().unwrap().0))
//!         .collect();
//!
//!     for (handle, health) in living {
//!         universe.get_entity_mut(handle)?.set(Health(health - 1))?;
//!     }
//!
//!     Ok(())
//! }
//!
//! fn remove_dead(universe: &mut Universe) -> Result<(), Box<dyn Error>> {
//!     let dead: Vec<Handle> = universe.find_signature(signature!(Health))
//!         .filter(|entity| entity.get::<Health>().unwrap().0 == 0)
//!         .map(|entity| entity.get_handle())
//!         .collect();
//!
//!     for handle in dead {
//!         universe.remove_entity(handle);
//!     }
//!
//!     Ok(())
//! }
//!
//! let mut universe = Universe::default();
//!
//! // Let's add a few Entities to our universe
//! let tank1_handle = universe.add_entity()
//!     .set(Health(2))
//!     .as_handle()?;
//!
//! let tank2_handle = universe.add_entity()
//!     .set(Health(1))
//!     .as_handle()?;
//!
//! damage_living(&mut universe)?;
//!
//! let tank1_health = universe.get_entity(tank1_handle)?.get::<Health>()?.0;
//! let tank2_health = universe.get_entity(tank2_handle)?.get::<Health>()?.0;
//! // Now that the transaction has been applied both tanks lose 1 heath
//! assert_eq!(tank1_health, 1);
//! assert_eq!(tank2_health, 0);
//!
//! remove_dead(&mut universe)?;
//!
//! assert!(universe.get_entity(tank2_handle).is_err());
//! #
//! # Ok::<(), Box<dyn Error>>(())
//! ```

mod attribute;
mod signature;
mod properties;
mod store;
mod universe;

pub use attribute::{Attribute, BoxedAttribute};
pub use signature::Signature;
pub use properties::{Property, Properties};
pub use store::{AttributeStore, Handle, HandleIterator, Query, QueryOne};
pub use universe::{AttributeIter, EntityBuilder, EntityMut, EntityRef, Universe};
