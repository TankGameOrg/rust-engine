use crate::rule::context::Context;
use crate::state::state::{Reference, State};

pub enum FailureType {
    PRECONDITION,
    RESOURCE(String),
    TIME(u64),
    OTHER,
}

pub struct ConditionFailure {
    message: String,
    failure: FailureType,
}

impl ConditionFailure {
    pub fn new(message: String, failure: FailureType) -> ConditionFailure {
        ConditionFailure { message, failure }
    }
}

pub type Precondition = Box<dyn Fn(&State, &Reference) -> Result<(), ConditionFailure>>;
pub type Condition = Box<dyn Fn(&Context) -> Result<(), ConditionFailure>>;
