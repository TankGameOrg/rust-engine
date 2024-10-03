use as_any::{AsAny, Downcast};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

#[typetag::serde(tag = "type")]
pub trait JsonType: AsAny + Debug {}

pub trait Container: JsonType {
    fn to_container(&self) -> Option<&AttributeContainer> {
        self.downcast_ref::<AttributeContainer>()
    }
}

pub struct Attribute<T: JsonType> {
    key: String,
    phantom_data: PhantomData<T>,
}

impl<T: JsonType> Attribute<T> {
    pub fn new(key: &str) -> Attribute<T> {
        Attribute {
            key: String::from(key),
            phantom_data: PhantomData {},
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttributeContainer {
    values: HashMap<String, Box<dyn JsonType>>,
    class: Option<String>,
}

impl AttributeContainer {
    pub fn new() -> AttributeContainer {
        AttributeContainer {
            values: HashMap::new(),
            class: None,
        }
    }

    pub fn new_with_class(string: String) -> AttributeContainer {
        AttributeContainer {
            values: HashMap::new(),
            class: Some(string),
        }
    }

    pub fn put<T: JsonType>(&mut self, attribute: &Attribute<T>, value: T) {
        self.values.insert(attribute.key.clone(), Box::new(value));
    }

    pub fn get<T: JsonType>(&self, attribute: &Attribute<T>) -> Option<&T> {
        match self.values.get(&attribute.key) {
            Some(value) => match value.as_ref().downcast_ref::<T>() {
                Some(t) => Some(t),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_unsafe<T: JsonType>(&self, attribute: &Attribute<T>) -> &T {
        self.get(attribute)
            .expect(format!("Failed to get from {:?} for {}", self, attribute.key).as_str())
    }

    /*
     * Returns an owned T instead of a &T. This clones the stored value, so T must impl Clone.
     */
    pub fn get_or_else<T: JsonType + Clone>(&self, attribute: &Attribute<T>, default: T) -> T {
        self.get(attribute)
            .map(|t| (*t).clone())
            .or(Some(default))
            .expect(format!("Failed to get from {:?} for {}", self, attribute.key).as_str())
    }

    pub fn has<T: JsonType>(&self, attribute: &Attribute<T>) -> bool {
        self.values.contains_key(&attribute.key)
    }
}

#[typetag::serde]
impl JsonType for AttributeContainer {}

impl Container for AttributeContainer {}
