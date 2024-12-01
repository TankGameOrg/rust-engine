use crate::rules::infrastructure::ecs::Attribute;

use super::StatusEffects;

/// A stat is a numeric value that represents an entity's proficiency in a skill
pub trait Stat: Attribute {
    /// Get the base value for this stat (not accounting for status effects)
    fn get_base(&self) -> usize;

    /// Set a new base value for this stat
    fn set_base(&mut self, base: usize);

    /// Get the value of this stat accounting for status effects
    fn get_current(&self) -> usize {
        self.get_status_effects().get_effected_value(self.get_base())
    }

    /// Get the status effects applied to this stat
    fn get_status_effects(&self) -> &StatusEffects;

    /// Get a mutable reference to the status effects applied to this stat
    fn get_status_effects_mut(&mut self) -> &mut StatusEffects;
}

#[macro_export]
macro_rules! generic_stat {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name {
            base: usize,
            status_effects: $crate::rules::base_game::attributes::StatusEffects,
        }

        impl $name {
            pub fn new(base: usize) -> $name {
                Self {
                    base,
                    status_effects: $crate::rules::base_game::attributes::StatusEffects::default(),
                }
            }
        }

        impl $crate::rules::infrastructure::ecs::Attribute for $name {}

        impl $crate::rules::base_game::attributes::Stat for $name {
            fn get_status_effects(&self) -> &$crate::rules::base_game::attributes::StatusEffects {
                &self.status_effects
            }

            fn get_status_effects_mut(&mut self) -> &mut $crate::rules::base_game::attributes::StatusEffects {
                &mut self.status_effects
            }

            fn get_base(&self) -> usize {
                self.base
            }

            fn set_base(&mut self, base: usize) {
                self.base = base;
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use $crate::rules::base_game::attributes::Stat;

                f.write_str("Stat { ")?;
                f.write_fmt(format_args!("base: {}, current: {}, status_effects: {:?} ", self.get_base(), self.get_current(), self.get_status_effects()))?;
                f.write_str(" }")
            }
        }
    };
}

generic_stat!(Range);
generic_stat!(Speed);

#[cfg(test)]
mod test {
    use crate::rules::base_game::attributes::{Effect, test::{TestStatusEffect, TestStatusEffect2}};

    use super::*;

    #[test]
    fn stat_test() {
        let mut range = Range::new(2);

        assert_eq!(range.get_current(), 2);

        range.get_status_effects_mut().add_effect(TestStatusEffect::new(Effect::Constant(1)));
        assert_eq!(range.get_current(), 3);

        range.get_status_effects_mut().add_effect(TestStatusEffect2::new(Effect::Percent(2.0)));
        assert_eq!(range.get_current(), 7);

        range.set_base(5);
        assert_eq!(range.get_current(), 16)
    }
}