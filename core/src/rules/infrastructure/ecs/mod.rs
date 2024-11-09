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
//!     for handle in universe.gather(&|handle| *universe.get_attribute(handle, &Health).unwrap_or(&0) > 0) {
//!         modify_entity!(&mut transaction, handle, {
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
//!     for handle in universe.gather(&|handle| *universe.get_attribute(handle, &Health).unwrap_or(&1) == 0) {
//!         transaction.add(RemoveEntityModification::new(handle));
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
//!     Health = 2
//! })?;
//!
//! let tank2_handle = create_entity_immidate!(&mut universe, {
//!     Health = 1
//! })?;
//!
//! let damage1_transaction = damage_living(&universe)?;
//!
//! let tank1_health = *universe.get_attribute(tank1_handle, &Health)?;
//! // Tank1 still has 2 heath because the transaction hasn't been applied
//! assert_eq!(tank1_health, 2);
//!
//! damage1_transaction.apply(&mut universe)?;
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
//!
//! In the previous example we use universe.gather() which searches all the entities in a universe and
//! returns the ones that match.  This becomes inefficient for large universes, to fix that we can define
//! an Index that tracks all entities with a specific attribute and provides optimized lookup functions for
//! that attribute.
//! ```
//! # use std::error::Error;
//! # use tank_game_core::*;
//! # use tank_game_core::rules::infrastructure::ecs::*;
//! # use tank_game_core::rules::infrastructure::transaction::*;
//! # use std::collections::HashSet;
//! #
//! // Let's start with an index that tracks all entities with health
//! struct LivingEntities {
//!     living: HashSet<Handle>, // health > 0
//!     dead: HashSet<Handle>,   // health == 0
//! }
//!
//! impl LivingEntities {
//!     fn new() -> LivingEntities {
//!         LivingEntities {
//!             living: HashSet::new(),
//!             dead: HashSet::new(),
//!         }
//!     }
//!
//!     // Each index defines it's own functions for finding/gathering entities by the attribute that it tracks
//!     fn gather_living<'iter>(&'iter self) -> impl Iterator<Item=Handle> + 'iter {
//!         self.living.iter().cloned()
//!     }
//!
//!     fn gather_dead<'iter>(&'iter self) -> impl Iterator<Item=Handle> + 'iter {
//!         self.dead.iter().cloned()
//!     }
//! }
//!
//! impl Index for LivingEntities {
//!     type AttributeValueType = u32;
//!
//!     fn add_attribute(
//!             &mut self,
//!             handle: Handle,
//!             new_value: &Self::AttributeValueType,
//!         ) -> Result<(), Box<dyn std::error::Error>> {
//!         // This method will only be called if the Health attribute is added to an entity so we don't have to
//!         // handle entities that don't have the Health attribute.
//!         if *new_value > 0 {
//!             self.living.insert(handle);
//!         }
//!         else {
//!             self.dead.insert(handle);
//!         }
//!
//!         Ok(())
//!     }
//!
//!     fn remove_attribute(
//!             &mut self,
//!             handle: Handle,
//!             old_value: &Self::AttributeValueType,
//!         ) -> Result<(), Box<dyn std::error::Error>> {
//!         if *old_value > 0 {
//!             self.living.remove(&handle);
//!         }
//!         else {
//!             self.dead.remove(&handle);
//!         }
//!
//!         Ok(())
//!     }
//!     
//!     // Since we haven't defined an update_attribute function if the Health attribute on an entity is modified
//!     // the default impl will call remove_attribute then add_attribute
//! }
//!
//! attribute!(Health: u32, indexed by LivingEntities);
//!
//! fn damage_living(universe: &Universe) -> Result<Transaction, Box<dyn Error>> {
//!     let mut transaction = Transaction::new();
//!
//!     for handle in universe.get_index(&Health).gather_living() {
//!         modify_entity!(&mut transaction, handle, {
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
//!     for handle in universe.get_index(&Health).gather_dead() {
//!         transaction.add(RemoveEntityModification::new(handle));
//!     }
//!
//!     Ok(transaction)
//! }
//!
//! let mut universe = Universe::new();
//! // After we construct our Universe we need to register LivingEntities as the index for Health
//! // so that the index knows to update it each time we modify the Health attribute.
//! // If you comment out this line LivingEntities::gather_* will fail with "Could not find an index for Health")
//! universe.add_index(&Health, LivingEntities::new());
//!
//! // The rest of this example will behave identically to the previous one
//! let tank1_handle = create_entity_immidate!(&mut universe, {
//!     Health = 2
//! })?;
//!
//! let tank2_handle = create_entity_immidate!(&mut universe, {
//!     Health = 1
//! })?;
//!
//! damage_living(&universe)?.apply(&mut universe)?;
//!
//! let tank1_health = *universe.get_attribute(tank1_handle, &Health)?;
//! let tank2_health = *universe.get_attribute(tank2_handle, &Health)?;
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
mod entity;
mod index;
mod universe;

pub use attribute::{AnyAttribute, Attribute, AttributeValue, IndexedBy};
pub use index::{Handle, Index};
pub use universe::Universe;
