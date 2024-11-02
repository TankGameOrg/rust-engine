use std::error::Error;

use super::{
    attribute::{Attribute, AttributeValue},
    pool::{Handle, Pool},
};

/// A modification to an attribute or entity
pub trait Modification {
    /// Modify the pool or one of it's Entities
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>>;
}

/// Modify an attribute on the entity referenced by the handle
pub struct AttributeModification<T: AttributeValue> {
    handle: Handle,
    attribute: &'static Attribute<T>,
    new_value: T,
}

impl<T: AttributeValue + Clone> AttributeModification<T> {
    #[inline]
    pub fn new(
        handle: Handle,
        attribute: &'static Attribute<T>,
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
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        let entity = pool.get_entity(self.handle)?;
        let current_value = entity
            .get(self.attribute)
            .ok()
            .map(|value| value.clone());

        if let Some(index) = pool.get_index_mut(self.attribute) {
            if let Some(current_value) = current_value {
                index.update_attribute_hook(self.handle, &current_value, &self.new_value)?;
            } else {
                index.add_attribute_hook(self.handle, &self.new_value)?;
            }
        }

        pool.get_entity_mut(self.handle)?
            .set(self.attribute, self.new_value.clone());
        Ok(())
    }
}

/// Remove an attribute from the entity
pub struct UnsetAttributeModification<T: AttributeValue> {
    handle: Handle,
    attribute: &'static Attribute<T>,
}

impl<T: AttributeValue> UnsetAttributeModification<T> {
    #[inline]
    pub fn new(handle: Handle, attribute: &'static Attribute<T>) -> UnsetAttributeModification<T> {
        UnsetAttributeModification {
            handle,
            attribute,
        }
    }
}

impl<T: AttributeValue + Clone> Modification for UnsetAttributeModification<T> {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        let entity = pool.get_entity(self.handle)?;
        let current_value = entity.get(self.attribute)?.clone();

        if let Some(index) = pool.get_index_mut(self.attribute) {
            index.remove_attribute_hook(self.handle, &current_value)?;
        }

        let entity = pool.get_entity_mut(self.handle)?;
        entity.remove(self.attribute);
        Ok(())
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
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        pool.add_entity(self.handle)?;
        Ok(())
    }
}

/// Remove a entity for the pool
pub struct RemoveEntityModification {
    handle: Handle,
}

impl RemoveEntityModification {
    #[inline]
    pub fn new(handle: Handle) -> RemoveEntityModification {
        RemoveEntityModification {
            handle,
        }
    }
}

impl Modification for RemoveEntityModification {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        let entity = pool.remove_entity(self.handle)?;

        for (attribute, attribute_value) in entity.iter() {
            if let Some(index) = pool.get_index_mut(attribute) {
                index.remove_attribute_hook(self.handle, attribute_value)?;
            }
        }

        Ok(())
    }
}

/// A series of modifications that can be applied to a pool
pub struct Transaction {
    modifications: Vec<Box<dyn Modification>>,
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
            let result = modification.apply(pool);

            if result.is_err() {
                pool.set_transaction_failed();
                return result;
            }
        }

        Ok(())
    }
}

/// Add the modifications required to create and initialize an Entity to the given transaction
///
/// ```
/// # use tank_game::rules::infrastructure::ecs::{Attribute, Transaction};
/// # use tank_game::create_entity;
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut transaction = Transaction::new();
/// let new_handle = create_entity!(&mut transaction, {
///     dummy_attribute = 3
/// });
/// ```
#[macro_export]
macro_rules! create_entity {
    ($transaction:expr, { $($($token:tt)*),+ }) => {
        {
            use $crate::modify_entity;

            let transaction: &mut $crate::rules::infrastructure::ecs::Transaction = $transaction;

            let (handle, new_entity_modification) = $crate::rules::infrastructure::ecs::CreateEntityModification::new();
            transaction.add(new_entity_modification);

            modify_entity!(transaction, handle, {
                $(
                    $($token)*
                ),+
            });

            handle
        }
    };
}

/// A helper for creating modifications to an Entity
///
/// ```
/// # use tank_game::rules::infrastructure::ecs::{Transaction, Attribute};
/// # use tank_game::{create_entity,modify_entity};
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut transaction = Transaction::new();
/// # let dummy_handle = create_entity!(&mut transaction, { dummy_attribute = 3 });
/// modify_entity!(&mut transaction, dummy_handle, {
///     dummy_attribute = 2
/// });
/// ```
/// 
/// You can also remove an attribute from a entity with unset
/// ```
/// # use tank_game::rules::infrastructure::ecs::{Transaction, Attribute};
/// # use tank_game::{create_entity,modify_entity};
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut transaction = Transaction::new();
/// # let dummy_handle = create_entity!(&mut transaction, { dummy_attribute = 3 });
/// modify_entity!(&mut transaction, dummy_handle, {
///     unset dummy_attribute
/// });
/// ```
#[macro_export]
macro_rules! modify_entity {
    ($transaction:expr, $handle:expr, { $($($token:tt)*),+ }) => {
        {
            use $crate::modify_attribute;

            let transaction: &mut $crate::rules::infrastructure::ecs::Transaction = $transaction;

            $(
                modify_attribute!(transaction, $handle, $($token)*);
            )+
        }
    };
}

/// A helper that creates modifications for a single attribute.  Most users should use modify_entity! or create_entity! instead.
#[macro_export]
macro_rules! modify_attribute {
    ($transaction:expr, $handle:expr, $attribute:ident = $value:expr) => {
        $transaction.add($crate::rules::infrastructure::ecs::AttributeModification::new($handle, &$attribute, $value));
    };

    ($transaction:expr, $handle:expr, unset $attribute:ident) => {
        $transaction.add($crate::rules::infrastructure::ecs::UnsetAttributeModification::new($handle, &$attribute));
    };
}

/// Like create_entity! but it creates a transaction and applies it immidately
/// 
/// ```
/// # use std::error::Error;
/// # use tank_game::rules::infrastructure::ecs::{Attribute, Pool};
/// # use tank_game::create_entity_immidate;
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut pool = Pool::new();
/// let new_handle = create_entity_immidate!(&mut pool, {
///     dummy_attribute = 3
/// })?;
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! create_entity_immidate {
    ($pool:expr, $($token:tt)*) => {
        {
            use $crate::create_entity;
            use $crate::rules::infrastructure::ecs::Transaction;

            let pool: &mut Pool = $pool;
            let mut transaction = Transaction::new();

            let handle = create_entity!(&mut transaction, $($token)*);

            match transaction.apply(pool) {
                Ok(()) => Ok(handle),
                Err(err) => Err(err),
            }
        }
    };
}

/// Like modify_entity! but it creates a transaction and applies it immidately
/// 
/// ```
/// # use std::error::Error;
/// # use tank_game::rules::infrastructure::ecs::{Attribute, Pool};
/// # use tank_game::{create_entity_immidate, modify_entity_immidate};
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut pool = Pool::new();
/// # let dummy_handle = create_entity_immidate!(&mut pool, { dummy_attribute = 3 })?;
/// modify_entity_immidate!(&mut pool, dummy_handle, {
///     dummy_attribute = 3
/// })?;
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! modify_entity_immidate {
    ($pool:expr, $($token:tt)*) => {
        {
            use $crate::modify_entity;
            use $crate::rules::infrastructure::ecs::Transaction;

            let pool: &mut Pool = $pool;
            let mut transaction = Transaction::new();

            modify_entity!(&mut transaction, $($token)*);

            transaction.apply(pool)
        }
    };
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::rules::infrastructure::{
        ecs::{attribute::DUMMY_ATTRIBUTE, pool::Index},
        RuleError,
    };

    use super::*;

    #[test]
    fn transaction_test() {
        let mut pool = Pool::new();

        let handle = create_entity_immidate!(&mut pool, { DUMMY_ATTRIBUTE = 2 }).unwrap();
        assert_eq!(*pool.get_entity(handle).unwrap().get(&DUMMY_ATTRIBUTE).unwrap(), 2);

        modify_entity_immidate!(&mut pool, handle, { unset DUMMY_ATTRIBUTE }).unwrap();
        assert!(pool.get_entity(handle).unwrap().get(&DUMMY_ATTRIBUTE).is_err());
    }

    struct TestIndex {
        handle: Option<Handle>,
    }

    impl TestIndex {
        fn new() -> TestIndex {
            return TestIndex { handle: None };
        }

        fn get<'iter>(pool: &'iter Pool) -> Result<Handle, Box<dyn Error>> {
            let index: &TestIndex = pool.get_index(&DUMMY_ATTRIBUTE)?;

            match index.handle {
                None => Err(Box::new(RuleError::Generic(String::from(
                    "No handle stored yet",
                )))),
                Some(handle) => Ok(handle),
            }
        }
    }

    impl Index for TestIndex {
        type AttributeValueType = u32;

        fn add_attribute(
            &mut self,
            handle: Handle,
            _new_value: &u32,
        ) -> Result<(), Box<dyn Error>> {
            self.handle = Some(handle);
            Ok(())
        }

        fn remove_attribute(
            &mut self,
            _handle: Handle,
            _old_value: &u32,
        ) -> Result<(), Box<dyn Error>> {
            self.handle = None;
            Ok(())
        }
    }

    #[test]
    fn index_test() {
        let mut pool = Pool::new();
        pool.add_index(&DUMMY_ATTRIBUTE, TestIndex::new());

        let handle = Handle::new();
        pool.add_entity(handle).unwrap();
        modify_entity_immidate!(&mut pool, handle, { DUMMY_ATTRIBUTE = 2 }).unwrap();

        pool.add_entity(Handle::new()).unwrap();

        let result = TestIndex::get(&pool).unwrap();
        assert_eq!(result, handle);

        let mut transaction = Transaction::new();
        transaction.add(RemoveEntityModification::new(handle));
        transaction.apply(&mut pool).unwrap();

        let result = TestIndex::get(&pool);
        assert!(result.is_err());
    }

    struct FailingIndex {}

    static mut FAILING_INDEX_FAILS: bool = false;

    impl FailingIndex {
        fn return_result(&self) -> Result<(), Box<dyn Error>> {
            if unsafe { FAILING_INDEX_FAILS } {
                Err(Box::new(RuleError::Generic(String::from("Tripped error"))))
            } else {
                Ok(())
            }
        }
    }

    impl Index for FailingIndex {
        type AttributeValueType = u32;

        fn add_attribute(
                &mut self,
                _handle: Handle,
                _new_value: &Self::AttributeValueType,
            ) -> Result<(), Box<dyn Error>> {
            self.return_result()
        }

        fn remove_attribute(
                &mut self,
                _handle: Handle,
                _old_value: &Self::AttributeValueType,
            ) -> Result<(), Box<dyn Error>> {
                self.return_result()
        }

        fn update_attribute(
                &mut self,
                _handle: Handle,
                _old_value: &Self::AttributeValueType,
                _new_value: &Self::AttributeValueType,
            ) -> Result<(), Box<dyn Error>> {
                self.return_result()
        }
    }

    #[test]
    fn failing_index() {
        let mut pool = Pool::new();
        pool.add_index(&DUMMY_ATTRIBUTE, FailingIndex {});

        let handle = Handle::new();
        pool.add_entity(handle).unwrap();

        let mut transaction = Transaction::new();
        modify_entity!(&mut transaction, handle, { DUMMY_ATTRIBUTE = 2 });

        transaction.apply(&mut pool).unwrap();

        unsafe { FAILING_INDEX_FAILS = true; }

        let mut transaction = Transaction::new();
        transaction.add(RemoveEntityModification::new(handle));
        assert!(transaction.apply(&mut pool).is_err());

        assert!(pool.get_entity(handle).is_err());
    }
}
