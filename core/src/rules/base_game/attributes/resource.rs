use std::error::Error;

use crate::rules::infrastructure::ecs::Attribute;

use super::StatusEffectOwner;

pub enum ResourceStatusEffect {
    Max,
}

/// An in game resource that is always between [0, max]
///
/// Resources are typically some kind of currency that can be aquired and spend
/// by actions.  While the value of the resource it's self is not effected by status
/// effects the max value can be.  If the max value ever drops below the current value
/// even because of a status effect the current value will be lowered to the max and
/// will not recover when the status effect goes away unless new resources are aquired.
#[must_use]
pub trait Resource: Attribute + StatusEffectOwner {
    /// Get the current value of the resource
    fn get_current(&self) -> usize;

    /// Get the max of the resource (including status effects)
    fn get_max(&self) -> usize;

    /// Get the base max of the resource (not including status effects)
    fn get_base_max(&self) -> usize;

    /// Modify the base max value of the resource
    fn set_base_max(&mut self, new_max: usize);

    /// Check if there is enough of a resource to spend it
    fn can_spend(&self, amount: usize) -> bool;

    /// Attempt to spend the resource and return the new resource or return an error
    fn spend(&mut self, amount: usize) -> Result<(), Box<dyn Error>>;

    /// Increase the resource's current value
    fn aquire(&mut self, amount: usize);
}

macro_rules! generic_resource {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name {
            current: usize,
            max: usize,
            max_effects: $crate::rules::base_game::attributes::StatusEffects,
        }

        impl $name {
            fn enforce_contraints(&mut self) {
                self.current = std::cmp::min(self.current, self.get_max());
            }

            /// Construct a new resource
            pub fn new(current: usize, max: usize) -> Self {
                Self {
                    current: std::cmp::min(current, max),
                    max,
                    max_effects:
                        $crate::rules::base_game::attributes::StatusEffects::default(),
                }
            }
        }

        impl $crate::rules::infrastructure::ecs::Attribute for $name {}

        impl Resource for $name {
            fn get_current(&self) -> usize {
                self.current
            }

            fn get_max(&self) -> usize {
                self.max_effects.get_effected_value(self.max)
            }

            fn get_base_max(&self) -> usize {
                self.max
            }

            fn can_spend(&self, amount: usize) -> bool {
                self.get_current() >= amount
            }

            fn spend(&mut self, amount: usize) -> Result<(), Box<dyn std::error::Error>>
            where
                Self: Sized,
            {
                if self.can_spend(amount) {
                    self.current -= amount;
                    self.enforce_contraints();
                    Ok(())
                } else {
                    Err($crate::basic_error!(
                        "You don't have enough {} to spend {}",
                        self.get_display_name(),
                        amount
                    ))
                }
            }

            fn aquire(&mut self, amount: usize)
            where
                Self: Sized,
            {
                self.current += amount;
                self.enforce_contraints();
            }

            fn set_base_max(&mut self, new_max: usize)
            where
                Self: Sized,
            {
                self.max = new_max;
                self.enforce_contraints();
            }
        }

        impl $crate::rules::base_game::attributes::StatusEffectOwner for $name {
            type StatusEffectId = $crate::rules::base_game::attributes::ResourceStatusEffect;
        
            fn get_status_effects(&self, effect_id: Self::StatusEffectId) -> &$crate::rules::base_game::attributes::StatusEffects {
                match effect_id {
                    $crate::rules::base_game::attributes::ResourceStatusEffect::Max => &self.max_effects,
                }
            }
        
            fn get_status_effects_mut(&mut self, effect_id: Self::StatusEffectId) -> &mut $crate::rules::base_game::attributes::StatusEffects {
                match effect_id {
                    $crate::rules::base_game::attributes::ResourceStatusEffect::Max => &mut self.max_effects,
                }
            }
        
            fn status_effects_updated(&mut self) {
                self.enforce_contraints();
            }
        }
    };
}

// Define some resouces that are commonly used across game versions
generic_resource!(Gold);
generic_resource!(Action);

#[cfg(test)]
mod test {
    use crate::rules::base_game::attributes::{Effect, TestStatusEffect};

    use super::*;

    #[test]
    fn test_resource_bounds_with_status_effetcs() {
        let mut gold = Gold::new(1, 2);

        gold.aquire(1);
        assert_eq!(gold.get_current(), 2);
        gold.spend(2).unwrap();
        assert_eq!(gold.get_current(), 0);

        gold.aquire(3);
        assert_eq!(gold.get_current(), 2);

        gold.add_effect(ResourceStatusEffect::Max, TestStatusEffect::new(
            Effect::Constant(-1),
        ));
        assert_eq!(gold.get_max(), 1);
        assert_eq!(gold.get_base_max(), 2);
        assert_eq!(gold.get_current(), 1);

        gold.aquire(3);
        assert_eq!(gold.get_current(), 1);
        gold.clear_effects(ResourceStatusEffect::Max);

        gold.aquire(3);
        assert_eq!(gold.get_current(), 2);

        gold.set_base_max(5);
        gold.aquire(2);
        assert_eq!(gold.get_current(), 4);
    }
}
