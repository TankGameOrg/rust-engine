/// The value that this effect applies
#[derive(Debug, Clone, Copy)]
pub enum NumericEffect {
    Constant(isize),
    Percent(f32),
}

impl NumericEffect {
    #[inline]
    pub fn get_effect_amount(&self, base: usize) -> isize {
        match self {
            Self::Constant(amount) => *amount,
            Self::Percent(percent) => (base as f32 * percent) as isize,
        }
    }
}

/// A status effect that can be applied to a numeric attribute
#[derive(Debug, Clone)]
pub struct NumericStatusEffect {
    name: &'static str,
    effect: NumericEffect,
}

impl NumericStatusEffect {
    #[inline]
    pub fn new(name: &'static str, effect: NumericEffect) -> NumericStatusEffect {
        NumericStatusEffect { name, effect }
    }

    #[inline]
    pub fn get_name(&self) -> &'static str {
        self.name
    }

    #[inline]
    pub fn get_effect(&self) -> NumericEffect {
        self.effect
    }
}

/// A list of status effects that are applied to one attribute or one part of an attribute i.e. the max
#[derive(Debug, Default, Clone)]
pub struct NumericStatusEffects {
    effects: Vec<NumericStatusEffect>,
}

impl NumericStatusEffects {
    /// Compute the value after status effects have been applied
    pub fn get_effected_value(&self, base: usize) -> usize {
        let effect: isize = self
            .effects
            .iter()
            .map(|effect| effect.effect.get_effect_amount(base))
            .sum();

        let mut effected = (base as isize) + effect;
        if effected < 0 {
            effected = 0;
        }

        effected as usize
    }

    pub fn add_effect(&mut self, effect: NumericStatusEffect) {
        self.effects.push(effect);
    }

    pub fn remove_effect(&mut self, effect_name: &'static str) {
        self.effects = self
            .effects
            .clone()
            .into_iter()
            .filter(|effect| effect.name != effect_name)
            .collect();
    }

    pub fn clear_effects(&mut self) {
        self.effects.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<NumericStatusEffect> {
        self.effects.iter()
    }
}

impl<'iter> IntoIterator for &'iter NumericStatusEffects {
    type Item = &'iter NumericStatusEffect;
    type IntoIter = std::slice::Iter<'iter, NumericStatusEffect>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut effects = NumericStatusEffects::default();
        effects.add_effect(NumericStatusEffect::new("foo", NumericEffect::Constant(1)));
        effects.add_effect(NumericStatusEffect::new("bar", NumericEffect::Percent(0.5)));

        assert_eq!(effects.get_effected_value(1), 2);
        assert_eq!(effects.get_effected_value(2), 4);

        effects.remove_effect("bar");

        assert_eq!(effects.get_effected_value(1), 2);
        assert_eq!(effects.get_effected_value(2), 3);
    }
}
