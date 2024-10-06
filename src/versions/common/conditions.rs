#[macro_export]
macro_rules! from_precondition {
    ($name:ident<$template:ident: $initial_constraint:ident $(+ $constraint:ident )*>($($names:ident: $types:ty),*)) => (
        fn $name<$template: $initial_constraint $(+ $constraint)*>($($names: $types),*) -> crate::rule::condition::Condition {
            Box::new(move |context: &crate::rule::context::Context| crate::versions::common::conditions::precondition::$name($($names),*)(context.state, &context.player))
        }
    )
}


// Preconditions
pub mod precondition {
    use crate::rule::condition::{ConditionFailure, FailureType, Precondition};
    use crate::util::attribute::{Attribute, JsonType};

    pub fn tank_has_attribute<T: JsonType>(attribute: &'static Attribute<T>) -> Precondition {
        Box::new(|state, player_ref| {
            if state.board().get_tank_for_ref(player_ref).map(|t| t.container().has(attribute)).unwrap_or(false) {
                Ok(())
            } else {
                Err(ConditionFailure::new(String::from(format!("Player tank does not exist or does not have an attribute: {}", attribute.key())), FailureType::RESOURCE(attribute.key().clone())))
            }
        })
    }

    pub fn tank_has_attribute_minimum<T: JsonType + PartialOrd>(attribute: &'static Attribute<T>, bound: T) -> Precondition {
        Box::new(move |state, player_ref| {
            if state.board().get_tank_for_ref(player_ref).map(|t| t.container().get(attribute).map(|t| *t >= bound).unwrap_or(false)).unwrap_or(false) {
                Ok(())
            } else {
                Err(ConditionFailure::new(String::from(format!("Player tank does not exist or does not have an attribute: {}", attribute.key())), FailureType::RESOURCE(attribute.key().clone())))
            }
        })
    }
}

pub mod condition {
    use crate::util::attribute::{Attribute, JsonType};

    from_precondition!(tank_has_attribute<T: JsonType>(attribute: &'static Attribute<T>));
    from_precondition!(tank_has_attribute_minimum<T: JsonType + PartialOrd + Copy>(attribute: &'static Attribute<T>, bound: T));
}