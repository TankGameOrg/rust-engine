use crate::rule::condition::{Condition, ConditionFailure, Precondition};
use crate::rule::context::Context;
use crate::state::state::{Reference, State};
use std::any::Any;
use std::collections::HashMap;

pub type Application = Box<dyn Fn(&mut Context)>;

// TODO figure out variants
pub type VariantProducer = Box<dyn Fn(&State, &Reference) -> Vec<Box<dyn Any>>>;

pub trait ActionProvider {
    fn actions(&self) -> HashMap<String, ActionDescription>;
}

pub struct ActionDescription {
    pub action: String,
    pub description: String,
    preconditions: Vec<&'static Precondition>,
    conditions: Vec<&'static Condition>,
    application: &'static Application,
    variants: HashMap<String, &'static VariantProducer>,
}

impl ActionDescription {
    pub fn new(
        action: String,
        description: String,
        preconditions: Vec<&'static Precondition>,
        conditions: Vec<&'static Condition>,
        application: &'static Application,
        variants: Vec<(String, &'static VariantProducer)>,
    ) -> ActionDescription {
        let variants_map: HashMap<String, &'static VariantProducer> =
            variants.iter().cloned().collect();
        ActionDescription {
            action,
            description,
            preconditions,
            conditions,
            application,
            variants: variants_map,
        }
    }

    pub fn check_preconditions(
        &self,
        state: &State,
        player_ref: &Reference,
    ) -> Vec<ConditionFailure> {
        self.preconditions
            .iter()
            .map(|f| f(state, player_ref))
            .filter_map(|f| f.err())
            .collect()
    }

    pub fn check_conditions(&self, context: &Context) -> Vec<ConditionFailure> {
        self.conditions
            .iter()
            .map(|f| f(context))
            .filter_map(|f| f.err())
            .collect()
    }

    pub fn apply(&self, context: &mut Context) {
        self.application.as_ref()(context)
    }

    pub fn produce_variants(&self, state: &State, player_ref: &Reference) -> Vec<()> {
        self.variants.iter().map(|(name, producer)| ()).collect()
    }
}
