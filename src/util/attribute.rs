use crate::state::board::Board;
use crate::state::position::Position;
use crate::state::state::Reference;
use crate::state::upgrade::Upgrade;
use as_any::{AsAny, Downcast};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

pub trait AttributeValue: AsAny + Debug + Send + Sync {}

pub struct Attribute<T: AttributeValue> {
    key: &'static str,
    phantom_data: PhantomData<T>,
}

impl<T: AttributeValue> Attribute<T> {
    pub fn new(key: &'static str) -> Attribute<T> {
        Attribute {
            key,
            phantom_data: PhantomData {},
        }
    }

    pub fn key(&self) -> &'static str {
        &self.key
    }
}

#[derive(Debug)]
pub struct AttributeContainer {
    values: HashMap<&'static str, Box<dyn AttributeValue>>,
}

impl AttributeContainer {
    pub fn new() -> AttributeContainer {
        AttributeContainer {
            values: HashMap::new(),
        }
    }

    pub fn put<T: AttributeValue>(&mut self, attribute: &Attribute<T>, value: T) {
        self.values.insert(attribute.key, Box::new(value));
    }

    pub fn get<T: AttributeValue>(&self, attribute: &Attribute<T>) -> Option<&T> {
        match self.values.get(attribute.key) {
            Some(value) => {
                Some(value.as_ref().downcast_ref::<T>().expect(
                    format!("Downcast error getting attribute `{}`", attribute.key).as_str(),
                ))
            }
            None => None,
        }
    }

    pub fn get_mut<T: AttributeValue>(&mut self, attribute: &Attribute<T>) -> Option<&mut T> {
        match self.values.get_mut(attribute.key) {
            Some(value) => {
                Some(value.as_mut().downcast_mut::<T>().expect(
                    format!("Downcast error getting attribute `{}`", attribute.key).as_str(),
                ))
            }
            None => None,
        }
    }

    pub fn get_unsafe<T: AttributeValue>(&self, attribute: &Attribute<T>) -> &T {
        let value = self.get(attribute);
        value.unwrap_or_else(|| panic!("Failed to get value for {}", attribute.key))
    }

    pub fn get_mut_unsafe<T: AttributeValue>(&mut self, attribute: &Attribute<T>) -> &mut T {
        self.get_mut(attribute)
            .unwrap_or_else(|| panic!("Failed to get value for {}", attribute.key))
    }

    /*
     * Returns an owned T instead of a &T. This clones the stored value, so T must impl Clone.
     */
    pub fn get_or_else<T: AttributeValue + Clone>(
        &self,
        attribute: &Attribute<T>,
        default: T,
    ) -> T {
        self.get(attribute).map(|t| (*t).clone()).unwrap_or(default)
    }

    pub fn has<T: AttributeValue>(&self, attribute: &Attribute<T>) -> bool {
        self.values.contains_key(&attribute.key)
    }
}

impl AttributeValue for bool {}
impl AttributeValue for i32 {}
impl AttributeValue for i64 {}
impl AttributeValue for String {}
impl AttributeValue for Reference {}
impl AttributeValue for Position {}
impl AttributeValue for Board {}

impl AttributeValue for Vec<Reference> {}
impl AttributeValue for Vec<String> {}
impl AttributeValue for Vec<Upgrade> {}
