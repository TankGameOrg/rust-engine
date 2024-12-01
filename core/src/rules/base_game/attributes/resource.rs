use std::error::Error;

use crate::{generic_stat, rules::infrastructure::ecs::Attribute};

generic_stat!(ResourceMax);

/// A finite in game resource that is always between [0, max]
pub trait Resource: Attribute {
    /// Get the current value of the resource
    fn get_current(&self) -> usize;

    /// Get the max amount of the resource
    fn get_max(&self) -> &ResourceMax;

    /// Modify the base max value of the resource
    fn set_max(&mut self, new_max: ResourceMax);
}

/// A Currency that can be aquired and spend by actions.  
pub trait Currency: Resource {
    /// Check if there is enough of a resource to spend it
    fn can_spend(&self, amount: usize) -> bool;

    /// Attempt to spend the resource and return the new resource or return an error
    fn spend(&mut self, amount: usize) -> Result<(), Box<dyn Error>>;

    /// Increase the resource's current value
    fn aquire(&mut self, amount: usize);
}

#[macro_export]
macro_rules! generic_resource {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name {
            current: usize,
            max: ResourceMax,
        }

        impl $name {
            fn enforce_contraints(&mut self) {
                use $crate::rules::base_game::attributes::Stat;

                self.current = std::cmp::min(self.current, self.get_max().get_current());
            }

            /// Construct a new resource
            pub fn new(current: usize, max: ResourceMax) -> Self {
                use $crate::rules::base_game::attributes::Stat;

                Self {
                    current: std::cmp::min(current, max.get_current()),
                    max,
                }
            }
        }

        impl $crate::rules::infrastructure::ecs::Attribute for $name {}

        impl $crate::rules::base_game::attributes::Resource for $name {
            fn get_current(&self) -> usize {
                self.current
            }

            fn get_max(&self) -> &ResourceMax {
                &self.max
            }

            fn set_max(&mut self, max: ResourceMax) {
                self.max = max;
                self.enforce_contraints();
            }

        }
    };
}

macro_rules! generic_currency {
    ($name:ident) => {
        $crate::generic_resource!($name);
        
        impl $crate::rules::base_game::attributes::Currency for $name {
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
        }
    };
}

// Define some resouces that are commonly used across game versions
generic_currency!(Gold);
generic_currency!(Action);

#[cfg(test)]
mod test {
    use crate::rules::base_game::attributes::{test::TestStatusEffect, Effect, Stat};

    use super::*;

    #[test]
    fn test_resource_bounds_with_status_effetcs() {
        let mut gold = Gold::new(1, ResourceMax::new(2));

        gold.aquire(1);
        assert_eq!(gold.get_current(), 2);
        gold.spend(2).unwrap();
        assert_eq!(gold.get_current(), 0);

        gold.aquire(3);
        assert_eq!(gold.get_current(), 2);

        let mut max_gold = gold.get_max().clone();
        max_gold.get_status_effects_mut().add_effect(TestStatusEffect::new(
            Effect::Constant(-1),
        ));
        gold.set_max(max_gold);

        println!("+ {:?}", gold);

        assert_eq!(gold.get_current(), 1);

        gold.aquire(3);
        assert_eq!(gold.get_current(), 1);

        let mut max_gold = gold.get_max().clone();
        max_gold.get_status_effects_mut().clear_effects();
        gold.set_max(max_gold);

        gold.aquire(3);
        assert_eq!(gold.get_current(), 2);

        gold.set_max(ResourceMax::new(5));
        gold.aquire(2);
        assert_eq!(gold.get_current(), 4);
    }
}
