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
    fn get_name(&self) -> &'static str;
    fn get_amount(&self) -> Effect;
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
            .map(|effect| effect.get_amount().get_effect_amount(base))
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

pub trait StatusEffectOwner {
    type StatusEffectId;

    fn get_status_effects(&self, effect_id: Self::StatusEffectId) -> &StatusEffects;
    fn get_status_effects_mut(&mut self, effect_id: Self::StatusEffectId) -> &mut StatusEffects;

    fn add_effect<T: StatusEffect>(&mut self, effect_id: Self::StatusEffectId, effect: T) {
        self.get_status_effects_mut(effect_id).add_effect(effect);
        self.status_effects_updated();
    }

    fn remove_effect<T: StatusEffect>(&mut self, effect_id: Self::StatusEffectId) {
        self.get_status_effects_mut(effect_id).remove_effect::<T>();
        self.status_effects_updated();
    }

    fn clear_effects(&mut self, effect_id: Self::StatusEffectId) {
        self.get_status_effects_mut(effect_id).clear_effects();
        self.status_effects_updated();
    }

    fn status_effects_updated(&mut self) {}
}

/// A status effect that can be used for testing status effect users
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct TestStatusEffect(Effect);

#[cfg(test)]
impl TestStatusEffect {
    pub fn new(effect: Effect) -> TestStatusEffect {
        TestStatusEffect(effect)
    }
}

#[cfg(test)]
impl StatusEffect for TestStatusEffect {
    fn get_name(&self) -> &'static str {
        "My test status effect"
    }
    
    fn get_amount(&self) -> Effect {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Clone)]
    struct FooEffect;

    impl StatusEffect for FooEffect {
        fn get_name(&self) -> &'static str {
            "My test effect"
        }

        fn get_amount(&self) -> Effect {
            Effect::Constant(1)
        }
    }

    #[derive(Debug, Clone)]
    struct BarEffect;

    impl StatusEffect for BarEffect {
        fn get_name(&self) -> &'static str {
            "My test effect"
        }

        fn get_amount(&self) -> Effect {
            Effect::Percent(0.5)
        }
    }

    #[test]
    fn test() {
        let mut effects = StatusEffects::default();
        effects.add_effect(FooEffect);
        effects.add_effect(BarEffect);

        assert_eq!(effects.get_effected_value(1), 2);
        assert_eq!(effects.get_effected_value(2), 4);

        effects.remove_effect::<BarEffect>();

        assert_eq!(effects.get_effected_value(1), 2);
        assert_eq!(effects.get_effected_value(2), 3);
    }
}
