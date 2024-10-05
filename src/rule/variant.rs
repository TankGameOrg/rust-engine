use crate::util::attribute::JsonType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Variant {
    pub attribute: String,
    pub possible_values: Vec<Box<dyn JsonType>>,
}

impl Variant {
    pub fn new(attribute: &String, possible_values: Vec<Box<dyn JsonType>>) -> Variant {
        Variant {
            attribute: attribute.clone(),
            possible_values,
        }
    }
}

#[typetag::serde]
impl JsonType for Variant {}
