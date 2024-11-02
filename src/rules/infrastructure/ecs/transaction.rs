use std::error::Error;

use super::{
    attribute::{Attribute, AttributeValue},
    pool::{Handle, Pool},
};

/// A modification to an attribute or container
pub trait Modification {
    /// Modify the pool or one of it's attribute containers
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>>;
}

/// Modify an attribute on the container referenced by the handle
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
        let container = pool.get_attribute_container(self.handle)?;
        let current_value = container
            .get(self.attribute)
            .ok()
            .map(|value| value.clone());

        if let Some(index) = pool.get_index_mut(self.attribute) {
            if let Some(current_value) = current_value {
                index.update_container_hook(self.handle, &current_value, &self.new_value)?;
            } else {
                index.add_container_hook(self.handle, &self.new_value)?;
            }
        }

        pool.get_attribute_container_mut(self.handle)?
            .set(self.attribute, self.new_value.clone());
        Ok(())
    }
}

/// Remove an attribute from the container
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
        let container = pool.get_attribute_container(self.handle)?;
        let current_value = container.get(self.attribute)?.clone();

        if let Some(index) = pool.get_index_mut(self.attribute) {
            index.remove_container_hook(self.handle, &current_value)?;
        }

        let container = pool.get_attribute_container_mut(self.handle)?;
        container.remove(self.attribute);
        Ok(())
    }
}

/// Create a new container that can be accessed with the given Handle
pub struct CreateContainerModification {
    handle: Handle,
}

impl CreateContainerModification {
    #[inline]
    pub fn new() -> (Handle, CreateContainerModification) {
        let handle = Handle::new();
        (handle, CreateContainerModification { handle })
    }
}

impl Modification for CreateContainerModification {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        pool.add_attribute_container(self.handle)?;
        Ok(())
    }
}

pub struct RemoveContainerModification {
    handle: Handle,
}

impl RemoveContainerModification {
    #[inline]
    pub fn new(handle: Handle) -> RemoveContainerModification {
        RemoveContainerModification {
            handle,
        }
    }
}

impl Modification for RemoveContainerModification {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        let container = pool.remove_container(self.handle)?;

        for (attribute, attribute_value) in container.iter() {
            if let Some(index) = pool.get_index_mut(attribute) {
                index.remove_container_hook(self.handle, attribute_value)?;
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

/// Add the modifications required to create and initialize an attribute container to the given transaction
///
/// ```
/// # use tank_game::rules::infrastructure::ecs::{Attribute, Transaction};
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
    ($transaction:expr, { $($($token:tt)*),+ }) => {
        {
            use $crate::modify_container;

            let transaction: &mut $crate::rules::infrastructure::ecs::Transaction = $transaction;

            let (handle, new_container_modification) = $crate::rules::infrastructure::ecs::CreateContainerModification::new();
            transaction.add(new_container_modification);

            modify_container!(transaction, handle, {
                $(
                    $($token)*
                ),+
            });

            handle
        }
    };
}

/// A helper for creating modifications to an attribute container
///
/// ```
/// # use tank_game::rules::infrastructure::ecs::{Transaction, Attribute};
/// # use tank_game::{create_container,modify_container};
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut transaction = Transaction::new();
/// # let dummy_handle = create_container!(&mut transaction, { dummy_attribute = 3 });
/// modify_container!(&mut transaction, dummy_handle, {
///     dummy_attribute = 2
/// });
/// ```
/// 
/// You can also remove an attribute from a container with unset
/// ```
/// # use tank_game::rules::infrastructure::ecs::{Transaction, Attribute};
/// # use tank_game::{create_container,modify_container};
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut transaction = Transaction::new();
/// # let dummy_handle = create_container!(&mut transaction, { dummy_attribute = 3 });
/// modify_container!(&mut transaction, dummy_handle, {
///     unset dummy_attribute
/// });
/// ```
#[macro_export]
macro_rules! modify_container {
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

/// A helper that creates modifications for a single attribute.  Most users should use modify_container! or create_container! instead.
#[macro_export]
macro_rules! modify_attribute {
    ($transaction:expr, $handle:expr, $attribute:ident = $value:expr) => {
        $transaction.add($crate::rules::infrastructure::ecs::AttributeModification::new($handle, &$attribute, $value));
    };

    ($transaction:expr, $handle:expr, unset $attribute:ident) => {
        $transaction.add($crate::rules::infrastructure::ecs::UnsetAttributeModification::new($handle, &$attribute));
    };
}

/// Like create_container! but it creates a transaction and applies it immidately
/// 
/// ```
/// # use std::error::Error;
/// # use tank_game::rules::infrastructure::ecs::{Attribute, Pool};
/// # use tank_game::create_container_immidate;
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut pool = Pool::new();
/// let new_handle = create_container_immidate!(&mut pool, {
///     dummy_attribute = 3
/// })?;
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! create_container_immidate {
    ($pool:expr, $($token:tt)*) => {
        {
            use $crate::create_container;
            use $crate::rules::infrastructure::ecs::Transaction;

            let pool: &mut Pool = $pool;
            let mut transaction = Transaction::new();

            let handle = create_container!(&mut transaction, $($token)*);

            match transaction.apply(pool) {
                Ok(()) => Ok(handle),
                Err(err) => Err(err),
            }
        }
    };
}

/// Like modify_container! but it creates a transaction and applies it immidately
/// 
/// ```
/// # use std::error::Error;
/// # use tank_game::rules::infrastructure::ecs::{Attribute, Pool};
/// # use tank_game::{create_container_immidate, modify_container_immidate};
/// # static dummy_attribute: Attribute<u32> = Attribute::<u32>::new("dummy_attribute");
/// #
/// let mut pool = Pool::new();
/// # let dummy_handle = create_container_immidate!(&mut pool, { dummy_attribute = 3 })?;
/// modify_container_immidate!(&mut pool, dummy_handle, {
///     dummy_attribute = 3
/// })?;
/// # Ok::<(), Box<dyn Error>>(())
/// ```
#[macro_export]
macro_rules! modify_container_immidate {
    ($pool:expr, $($token:tt)*) => {
        {
            use $crate::modify_container;
            use $crate::rules::infrastructure::ecs::Transaction;

            let pool: &mut Pool = $pool;
            let mut transaction = Transaction::new();

            modify_container!(&mut transaction, $($token)*);

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

        let handle = create_container_immidate!(&mut pool, { DUMMY_ATTRIBUTE = 2 }).unwrap();
        assert_eq!(*pool.get_attribute_container(handle).unwrap().get(&DUMMY_ATTRIBUTE).unwrap(), 2);

        modify_container_immidate!(&mut pool, handle, { unset DUMMY_ATTRIBUTE }).unwrap();
        assert!(pool.get_attribute_container(handle).unwrap().get(&DUMMY_ATTRIBUTE).is_err());
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

        fn add_container(
            &mut self,
            handle: Handle,
            _new_value: &u32,
        ) -> Result<(), Box<dyn Error>> {
            self.handle = Some(handle);
            Ok(())
        }

        fn remove_container(
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
        pool.add_attribute_container(handle).unwrap();
        modify_container_immidate!(&mut pool, handle, { DUMMY_ATTRIBUTE = 2 }).unwrap();

        pool.add_attribute_container(Handle::new()).unwrap();

        let result = TestIndex::get(&pool).unwrap();
        assert_eq!(result, handle);

        let mut transaction = Transaction::new();
        transaction.add(RemoveContainerModification::new(handle));
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

        fn add_container(
                &mut self,
                _handle: Handle,
                _new_value: &Self::AttributeValueType,
            ) -> Result<(), Box<dyn Error>> {
            self.return_result()
        }

        fn remove_container(
                &mut self,
                _handle: Handle,
                _old_value: &Self::AttributeValueType,
            ) -> Result<(), Box<dyn Error>> {
                self.return_result()
        }

        fn update_container(
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
        pool.add_attribute_container(handle).unwrap();

        let mut transaction = Transaction::new();
        modify_container!(&mut transaction, handle, { DUMMY_ATTRIBUTE = 2 });

        transaction.apply(&mut pool).unwrap();

        unsafe { FAILING_INDEX_FAILS = true; }

        let mut transaction = Transaction::new();
        transaction.add(RemoveContainerModification::new(handle));
        assert!(transaction.apply(&mut pool).is_err());

        assert!(pool.get_attribute_container(handle).is_err());
    }
}
