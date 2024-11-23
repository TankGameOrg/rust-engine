//! The entity component system used to represent all of the objects in the game
//!
//! Here's an example of how to create a universe and add some basic entities to it
//! ```
//! # use std::error::Error;
//! # use tank_game_core::*;
//! # use tank_game_core::rules::infrastructure::ecs::*;
//! # use tank_game_core::rules::infrastructure::transaction::*;
//! #
//! attribute!(Health: u32);
//!
//! fn damage_living(universe: &Universe) -> Result<Transaction, Box<dyn Error>> {
//!     let mut transaction = Transaction::new();
//!
//!     let living = universe.gather(signature!(Health))
//!         .filter(|handle| *universe.get_attribute(*handle, &Health).unwrap() > 0);
//! 
//!     for handle in living {
//!         modify_entity_transaction!(&mut transaction, handle, {
//!             Health = universe.get_attribute(handle, &Health)? - 1
//!         });
//!     }
//!
//!     Ok(transaction)
//! }
//!
//! fn remove_dead(universe: &Universe) -> Result<Transaction, Box<dyn Error>> {
//!     let mut transaction = Transaction::new();
//!
//!     let dead = universe.gather(signature!(Health))
//!         .filter(|handle| *universe.get_attribute(*handle, &Health).unwrap_or(&1) == 0);
//! 
//!     for handle in dead {
//!         transaction.add(RemoveEntityModification::new(handle));
//!     }
//!
//!     Ok(transaction)
//! }
//!
//! let mut universe = Universe::new();
//!
//! // Let's add a few Entities to our universe
//! let tank1_handle = create_entity!(&mut universe, {
//!     Health = 2
//! })?;
//!
//! let tank2_handle = create_entity!(&mut universe, {
//!     Health = 1
//! })?;
//!
//! let damage1_transactionaction = damage_living(&universe)?;
//!
//! let tank1_health = *universe.get_attribute(tank1_handle, &Health)?;
//! // Tank1 still has 2 heath because the transaction hasn't been applied
//! assert_eq!(tank1_health, 2);
//!
//! damage1_transactionaction.apply(&mut universe)?;
//!
//! let tank1_health = *universe.get_attribute(tank1_handle, &Health)?;
//! let tank2_health = *universe.get_attribute(tank2_handle, &Health)?;
//! // Now that the transaction has been applied both tanks lose 1 heath
//! assert_eq!(tank1_health, 1);
//! assert_eq!(tank2_health, 0);
//!
//! remove_dead(&universe)?.apply(&mut universe)?;
//!
//! assert!(universe.get_attribute(tank2_handle, &Health).is_err());
//! #
//! # Ok::<(), Box<dyn Error>>(())
//! ```

mod attribute;
mod signature;
mod store;
mod universe;

pub use signature::Signature;
pub use attribute::{AnyAttribute, Attribute, AttributeValue, FlagAttribute};
pub use store::Handle;
pub use universe::Universe;
