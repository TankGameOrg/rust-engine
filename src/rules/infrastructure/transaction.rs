use std::error::Error;

use super::{attribute::{Attribute, AttributeValue}, pool::{Handle, Pool}};

pub trait Modification {
    /// Modify the pool or one of it's attribute containers
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>>;
}

/// Modify an attribute on the container referenced by the handle
pub struct AttributeModification<T: AttributeValue> {
    handle: Handle,
    attribute: &'static Attribute<T>,
    new_value: T
}

impl<T: AttributeValue + Clone> AttributeModification<T> {
    #[inline]
    pub fn new(handle: Handle, attribute: &'static Attribute<T>, new_value: T) -> AttributeModification<T> {
        AttributeModification {
            handle,
            attribute,
            new_value,
        }
    }
}

impl<T: AttributeValue + Clone> Modification for AttributeModification<T> {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        let container = pool.get_attribute_container(self.handle)?;
        let current_value = container.get(self.attribute)
            .ok()
            .map(|value| value.clone());

        if let Some(index) = pool.get_index_mut(self.attribute) {
            if let Some(current_value) = current_value {
                index.update_container(self.handle, &current_value, &self.new_value);
            }
            else {
                index.add_container(self.handle, &self.new_value);
            }
        }

        pool.get_attribute_container_mut(self.handle)?.set(self.attribute, self.new_value.clone());
        Ok(())
    }
}

/// Create a new container that can be accessed with the given Handle
pub struct AddContainerModification {
    handle: Handle,
}

impl AddContainerModification {
    #[inline]
    pub fn new() -> (Handle, AddContainerModification) {
        let handle = Handle::new();
        (handle, AddContainerModification {
            handle,
        })
    }
}

impl Modification for AddContainerModification {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        pool.add_attribute_container_with_handle(self.handle)?;
        Ok(())
    }
}

/// A series of modifications that can be applied to a pool
pub struct Transaction {
    modifications: Vec<Box<dyn Modification>>
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
    pub fn apply(self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        for modification in self.modifications {
            modification.apply(pool)?;
        }

        Ok(())
    }
}

/// Add the modifications required to create and initialize an attribute container to the given transaction
///
/// ```
/// # use tank_game::rules::infrastructure::attribute::Attribute;
/// # use tank_game::rules::infrastructure::transaction::Transaction;
/// # use tank_game::create_container;
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut transaction = Transaction::new();
/// let new_handle = create_container!(&mut transaction, {
///     dummy_attribute = 3
/// });
/// ```
#[macro_export]
macro_rules! create_container {
    ($transaction:expr, { $($attribute:ident = $value:expr),+ }) => {
        {
            use $crate::modify_container;

            let transaction: &mut $crate::rules::infrastructure::transaction::Transaction = $transaction;

            let (handle, new_container_modification) = $crate::rules::infrastructure::transaction::AddContainerModification::new();
            transaction.add(new_container_modification);

            modify_container!(transaction, handle, {
                $(
                    $attribute = $value
                ),+
            });

            handle
        }
    };
}

/// A helper for creating modifications to an attribute container
///
/// ```
/// # use tank_game::rules::infrastructure::attribute::Attribute;
/// # use tank_game::rules::infrastructure::transaction::Transaction;
/// # use tank_game::{create_container,modify_container};
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut transaction = Transaction::new();
/// # let dummy_handle = create_container!(&mut transaction, { dummy_attribute = 3 });
/// modify_container!(&mut transaction, dummy_handle, {
///     dummy_attribute = 2
/// });
/// ```
#[macro_export]
macro_rules! modify_container {
    ($transaction:expr, $handle:expr, { $($attribute:ident = $value:expr),+ }) => {
        {
            let transaction: &mut $crate::rules::infrastructure::transaction::Transaction = $transaction;

            $(
                transaction.add($crate::rules::infrastructure::transaction::AttributeModification::new($handle, &$attribute, $value));
            )+
        }
    };
}