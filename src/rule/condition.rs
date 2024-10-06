use crate::rule::context::Context;
use crate::state::meta::player::PlayerRef;
use crate::state::state::State;
use crate::util::attribute::JsonType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum FailureType {
    PRECONDITION,
    RESOURCE(String),
    TIME(u64),
    OTHER,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConditionFailure {
    message: String,
    failure: FailureType,
}

impl ConditionFailure {
    pub fn new(message: String, failure: FailureType) -> ConditionFailure {
        ConditionFailure { message, failure }
    }
}

#[typetag::serde]
impl JsonType for FailureType {}

#[typetag::serde]
impl JsonType for ConditionFailure {}

pub type Precondition = Box<dyn Fn(&State, &PlayerRef) -> Result<(), ConditionFailure>>;
pub type Condition = Box<dyn Fn(&Context) -> Result<(), ConditionFailure>>;
