use std::{any::TypeId, collections::HashMap};

/// The value that this effect applies
#[derive(Debug, Clone, Copy)]
pub enum Effect {
    Constant(isize),
    Percent(f32),
}

impl Effect {
    #[inline]
    pub fn get_effect_amount(&self, base: usize) -> isize {
        match self {
            Self::Constant(amount) => *amount,
            Self::Percent(percent) => (base as f32 * percent) as isize,
        }
    }
}

/// A status effect that can be applied to a numeric attribute
pub trait StatusEffect: std::fmt::Debug + Send + Sync + StatusEffectClone + 'static {
    /// Get a human readable name for this status effect
    fn get_name(&self) -> &'static str;

    /// Get the [`Effect`] that this status effect applies
    fn get_effect(&self) -> Effect;
}

/// Clone for StatusEffect implementors
pub trait StatusEffectClone {
    fn clone_effect(&self) -> Box<dyn StatusEffect>;
}

impl<T: StatusEffect + Clone> StatusEffectClone for T {
    fn clone_effect(&self) -> Box<dyn StatusEffect> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn StatusEffect> {
    fn clone(&self) -> Self {
        self.clone_effect()
    }
}

/// A list of status effects that are applied to one attribute or one part of an attribute i.e. the max
#[derive(Debug, Default, Clone)]
pub struct StatusEffects {
    effects: HashMap<TypeId, Box<dyn StatusEffect>>,
}

impl StatusEffects {
    /// Compute the value after status effects have been applied
    pub fn get_effected_value(&self, base: usize) -> usize {
        let effect: isize = self
            .effects
            .values()
            .map(|effect| effect.get_effect().get_effect_amount(base))
            .sum();

        let mut effected = (base as isize) + effect;
        if effected < 0 {
            effected = 0;
        }

        effected as usize
    }

    pub fn add_effect<T: StatusEffect>(&mut self, effect: T) {
        self.effects.insert(TypeId::of::<T>(), Box::new(effect));
    }

    pub fn remove_effect<T: StatusEffect>(&mut self) {
        self.effects.remove(&TypeId::of::<T>());
    }

    pub fn clear_effects(&mut self) {
        self.effects.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn StatusEffect> {
        self.effects.values().map(|modification| modification.as_ref())
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    macro_rules! test_status_effect {
        ($name:ident) => {
            /// A status effect that can be used for testing status effect users
            #[derive(Debug, Clone)]
            pub struct $name(Effect);

            impl $name {
                pub fn new(effect: Effect) -> Self {
                    Self(effect)
                }
            }

            impl StatusEffect for $name {
                fn get_name(&self) -> &'static str {
                    "My test status effect"
                }
                
                fn get_effect(&self) -> Effect {
                    self.0
                }
            }
        };
    }

    test_status_effect!(TestStatusEffect);
    test_status_effect!(TestStatusEffect2);

    #[test]
    fn test() {
        let mut effects = StatusEffects::default();
        effects.add_effect(TestStatusEffect::new(Effect::Constant(1)));
        effects.add_effect(TestStatusEffect2::new(Effect::Percent(0.5)));

        assert_eq!(effects.get_effected_value(1), 2);
        assert_eq!(effects.get_effected_value(2), 4);

        effects.remove_effect::<TestStatusEffect2>();

        assert_eq!(effects.get_effected_value(1), 2);
        assert_eq!(effects.get_effected_value(2), 3);
    }
}
