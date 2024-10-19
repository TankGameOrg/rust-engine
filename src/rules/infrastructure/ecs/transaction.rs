use std::error::Error;

use super::{attribute::Attribute, container::AttributeContainer, pool::{Handle, Pool}};

pub trait Modification {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>>;
}

pub struct AttributeModification<T: 'static + Clone> {
    handle: Handle,
    attribute: &'static Attribute<T>,
    new_value: T
}

impl<T: 'static + Clone> AttributeModification<T> {
    pub fn new(handle: Handle, attribute: &'static Attribute<T>, new_value: T) -> AttributeModification<T> {
        AttributeModification {
            handle,
            attribute,
            new_value,
        }
    }
}

impl<T: 'static + Clone> Modification for AttributeModification<T> {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        pool.get_attribute_container_mut(self.handle)?.set(self.attribute, self.new_value.clone());
        Ok(())
    }
}

pub struct AddContainerModification {
    handle: Handle,
}

impl AddContainerModification {
    pub fn new() -> (Handle, AddContainerModification) {
        let handle = Handle::new();
        (handle, AddContainerModification {
            handle,
        })
    }
}

impl Modification for AddContainerModification {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        pool.add_attribute_container(self.handle, AttributeContainer::new());
        Ok(())
    }
}

pub struct Transaction {
    modifications: Vec<Box<dyn Modification>>
}

impl Transaction {
    pub fn new() -> Transaction {
        Transaction {
            modifications: Vec::new(),
        }
    }

    pub fn add<T: Modification + 'static>(&mut self, modification: T) {
        self.modifications.push(Box::new(modification));
    }

    pub fn apply(self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        for modification in self.modifications {
            modification.apply(pool)?;
        }

        Ok(())
    }
}


#[macro_export]
macro_rules! create_container {
    ($transaction:expr, { $($attribute:ident = $value:expr),+ }) => {
        {
            let transaction: &mut $crate::rules::infrastructure::ecs::transaction::Transaction = $transaction;

            let (handle, new_container_modification) = $crate::rules::infrastructure::ecs::transaction::AddContainerModification::new();
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


#[macro_export]
macro_rules! modify_container {
    ($transaction:expr, $handle:expr, { $($attribute:ident = $value:expr),+ }) => {
        {
            let transaction: &mut $crate::rules::infrastructure::ecs::transaction::Transaction = $transaction;

            $(
                transaction.add($crate::rules::infrastructure::ecs::transaction::AttributeModification::new($handle, &$attribute, $value));
            )+
        }
    };
}