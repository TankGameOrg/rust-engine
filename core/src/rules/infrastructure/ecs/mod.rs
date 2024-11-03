//! The entity component system used to represent all of the objects in the game
//!
//! ```
//! # use std::error::Error;
//! # use tank_game::*;
//! # use tank_game::rules::infrastructure::ecs::*;
//! #
//! attribute!(HEALTH: u32);
//! attribute!(REGEN: u32);
//!
//! fn damage_living(universe: &Universe) -> Result<Transaction, Box<dyn Error>> {
//!     let mut transaction = Transaction::new();
//!
//!     for pair in universe.gather(&|entity| *entity.get(&HEALTH).or::<&u32>(Ok(&0)).unwrap() > 0)? {
//!         modify_entity!(&mut transaction, pair.handle, {
//!             HEALTH = pair.entity.get(&HEALTH)? - 1
//!         });
//!     }
//!
//!     Ok(transaction)
//! }
//!
//! fn remove_dead(universe: &Universe) -> Result<Transaction, Box<dyn Error>> {
//!     let mut transaction = Transaction::new();
//!
//!     for pair in universe.gather(&|entity| *entity.get(&HEALTH).or::<&u32>(Ok(&1)).unwrap() == 0)? {
//!         transaction.add(RemoveEntityModification::new(pair.handle));
//!     }
//!
//!     Ok(transaction)
//! }
//!
//! let mut universe = Universe::new();
//!
//! // Let's add a few Entities to our universe
//! // Unlike the normal create_entity!() create_entity_immidate!() operates directly on the universe
//! // and applies immidatly
//! let tank1_handle = create_entity_immidate!(&mut universe, {
//!     HEALTH = 2
//! })?;
//!
//! let tank2_handle = create_entity_immidate!(&mut universe, {
//!     HEALTH = 1,
//!     REGEN = 1
//! })?;
//!
//! let damage1_transaction = damage_living(&universe)?;
//!
//! let tank1_health = *universe.get_entity(tank1_handle)?.get(&HEALTH)?;
//! // Tank1 still has 2 heath because the transaction hasn't been applied
//! assert_eq!(tank1_health, 2);
//!
//! damage1_transaction.apply(&mut universe)?;
//!
//! let tank1_health = *universe.get_entity(tank1_handle)?.get(&HEALTH)?;
//! let tank2_health = *universe.get_entity(tank2_handle)?.get(&HEALTH)?;
//! // Now that the transaction has been applied both tanks lose 1 heath
//! assert_eq!(tank1_health, 1);
//! assert_eq!(tank2_health, 0);
//!
//! remove_dead(&universe)?.apply(&mut universe)?;
//!
//! assert!(universe.get_entity(tank2_handle).is_err());
//! #
//! # Ok::<(), Box<dyn Error>>(())
//! ```

mod attribute;
mod entity;
mod transaction;
mod universe;

pub use attribute::{AnyAttribute, Attribute, AttributeValue};
pub use entity::Entity;
pub use transaction::*;
pub use universe::{GatheredResult, Handle, Index, Universe};
