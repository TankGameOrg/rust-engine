use std::error::Error;

use super::ecs::{Attribute, AttributeValue, Handle, Universe};

/// A modification to an attribute or entity
pub trait Modification {
    /// Modify the universe or one of it's Entities
    fn apply(&self, universe: &mut Universe) -> Result<(), Box<dyn Error>>;
}

/// Modify an attribute on the entity referenced by the handle
pub struct AttributeModification<T: AttributeValue> {
    handle: Handle,
    attribute: &'static dyn Attribute<T>,
    new_value: T,
}

impl<T: AttributeValue + Clone> AttributeModification<T> {
    #[inline]
    pub fn new(
        handle: Handle,
        attribute: &'static dyn Attribute<T>,
        new_value: T,
    ) -> AttributeModification<T> {
        AttributeModification {
            handle,
            attribute,
            new_value,
        }
    }
}

impl<T: AttributeValue + Clone> Modification for AttributeModification<T> {
    fn apply(&self, universe: &mut Universe) -> Result<(), Box<dyn Error>> {
        universe.set_attribute(self.handle, self.attribute, self.new_value.clone())
    }
}

/// Remove an attribute from the entity
pub struct UnsetAttributeModification<T: AttributeValue> {
    handle: Handle,
    attribute: &'static dyn Attribute<T>,
}

impl<T: AttributeValue> UnsetAttributeModification<T> {
    #[inline]
    pub fn new(
        handle: Handle,
        attribute: &'static dyn Attribute<T>,
    ) -> UnsetAttributeModification<T> {
        UnsetAttributeModification { handle, attribute }
    }
}

impl<T: AttributeValue> Modification for UnsetAttributeModification<T> {
    fn apply(&self, universe: &mut Universe) -> Result<(), Box<dyn Error>> {
        universe.remove_attribute(self.handle, self.attribute)
    }
}

/// Create a new entity that can be accessed with the given Handle
pub struct CreateEntityModification {
    handle: Handle,
}

impl CreateEntityModification {
    #[inline]
    pub fn new() -> (Handle, CreateEntityModification) {
        let handle = Handle::new();
        (handle, CreateEntityModification { handle })
    }
}

impl Modification for CreateEntityModification {
    fn apply(&self, universe: &mut Universe) -> Result<(), Box<dyn Error>> {
        universe.add_entity(self.handle)?;
        Ok(())
    }
}

/// Remove a entity for the universe
pub struct RemoveEntityModification {
    handle: Handle,
}

impl RemoveEntityModification {
    #[inline]
    pub fn new(handle: Handle) -> RemoveEntityModification {
        RemoveEntityModification { handle }
    }
}

impl Modification for RemoveEntityModification {
    fn apply(&self, universe: &mut Universe) -> Result<(), Box<dyn Error>> {
        universe.remove_entity(self.handle)
    }
}

/// A series of modifications that can be applied to a universe
pub struct Transaction {
    modifications: Vec<Box<dyn Modification>>,
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}

impl Transaction {
    #[inline]
    pub fn new() -> Transaction {
        Transaction {
            modifications: Vec::new(),
        }
    }

    /// Add a modification to this transaction
    #[inline]
    pub fn add<T: Modification + 'static>(&mut self, modification: T) {
        self.modifications.push(Box::new(modification));
    }

    /// Apply the modifications in the order they were added in
    #[inline]
    pub fn apply(self, universe: &mut Universe) -> Result<(), Box<dyn Error>> {
        for modification in self.modifications {
            modification.apply(universe)?;
        }

        Ok(())
    }
}

/// Add the modifications required to create and initialize an Entity to the given transaction
///
/// ```
/// # use tank_game_core::attribute;
/// # use tank_game_core::rules::infrastructure::ecs::Attribute;
/// # use tank_game_core::rules::infrastructure::transaction::Transaction;
/// # use tank_game_core::create_entity;
/// # attribute!(DummyAttribute: u32);
/// #
/// let mut transaction = Transaction::new();
/// let new_handle = create_entity!(&mut transaction, {
///     DummyAttribute = 3
/// });
/// ```
#[macro_export]
macro_rules! create_entity {
    ($transaction:expr, { $($($attribute:ident = $value:expr)*),+ }) => {
        {
            use $crate::modify_entity;

            let transaction: &mut $crate::rules::infrastructure::transaction::Transaction = $transaction;

            let (handle, new_entity_modification) = $crate::rules::infrastructure::transaction::CreateEntityModification::new();
            transaction.add(new_entity_modification);

            modify_entity!(transaction, handle, {
                $(
                    $($attribute = $value)*
                ),+
            });

            handle
        }
    };
}

/// A helper for creating modifications to an Entity
///
/// ```
/// # use tank_game_core::attribute;
/// # use tank_game_core::rules::infrastructure::ecs::Attribute;
/// # use tank_game_core::rules::infrastructure::transaction::Transaction;
/// # use tank_game_core::{create_entity,modify_entity};
/// # attribute!(DummyAttribute: u32);
/// #
/// let mut transaction = Transaction::new();
/// # let dummy_handle = create_entity!(&mut transaction, { DummyAttribute = 3 });
/// modify_entity!(&mut transaction, dummy_handle, {
///     DummyAttribute = 2
/// });
/// ```
#[macro_export]
macro_rules! modify_entity {
    ($transaction:expr, $handle:expr, { $($attribute:ident = $value:expr),+ }) => {
        {
            let transaction: &mut $crate::rules::infrastructure::transaction::Transaction = $transaction;

            $(
                transaction.add($crate::rules::infrastructure::transaction::AttributeModification::new($handle, &$attribute, $value));
            )+
        }
    };
}

/// Like create_entity! but it creates a transaction and applies it immidately
///
/// ```
/// # use tank_game_core::attribute;
/// # use std::error::Error;
/// # use tank_game_core::rules::infrastructure::ecs::{Attribute, Universe};
/// # use tank_game_core::create_entity_immidate;
/// # attribute!(DummyAttribute: u32);
/// #
/// let mut universe = Universe::new();
/// let new_handle = create_entity_immidate!(&mut universe, {
///     DummyAttribute = 3
/// })?;
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! create_entity_immidate {
    ($universe:expr, $($token:tt)*) => {
        {
            use $crate::create_entity;
            use $crate::rules::infrastructure::transaction::Transaction;

            let universe: &mut Universe = $universe;
            let mut transaction = Transaction::new();

            let handle = create_entity!(&mut transaction, $($token)*);

            match transaction.apply(universe) {
                Ok(()) => Ok(handle),
                Err(err) => Err(err),
            }
        }
    };
}

/// Like modify_entity! but it creates a transaction and applies it immidately
///
/// ```
/// # use tank_game_core::attribute;
/// # use std::error::Error;
/// # use tank_game_core::rules::infrastructure::ecs::{Attribute, Universe};
/// # use tank_game_core::{create_entity_immidate, modify_entity_immidate};
/// # attribute!(DummyAttribute: u32);
/// #
/// let mut universe = Universe::new();
/// # let dummy_handle = create_entity_immidate!(&mut universe, { DummyAttribute = 3 })?;
/// modify_entity_immidate!(&mut universe, dummy_handle, {
///     DummyAttribute = 3
/// })?;
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! modify_entity_immidate {
    ($universe:expr, $($token:tt)*) => {
        {
            use $crate::modify_entity;
            use $crate::rules::infrastructure::transaction::Transaction;

            let universe: &mut Universe = $universe;
            let mut transaction = Transaction::new();

            modify_entity!(&mut transaction, $($token)*);

            transaction.apply(universe)
        }
    };
}

#[cfg(test)]
mod test {
    use crate::attribute;

    use super::*;

    attribute!(DummyAttribute: u32);

    #[test]
    fn transaction_test() {
        let mut universe = Universe::new();

        let handle = create_entity_immidate!(&mut universe, { DummyAttribute = 2 }).unwrap();
        assert_eq!(*universe.get_attribute(handle, &DummyAttribute).unwrap(), 2);

        let mut transaction = Transaction::new();
        transaction.add(UnsetAttributeModification::new(handle, &DummyAttribute));
        transaction.apply(&mut universe).unwrap();
        assert!(universe.get_attribute(handle, &DummyAttribute).is_err());
    }
}
