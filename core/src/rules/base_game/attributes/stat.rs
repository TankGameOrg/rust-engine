use crate::rules::infrastructure::ecs::Attribute;

/// A stat generally represents an entity's proficiency in a skill
/// 
/// Specifically it is a numeric attribute with a min, current, and max
/// where current is bounded by [min, max] and all three can have status
/// effects applied to them.
pub trait Stat: Attribute {
    fn get_min(&self) -> usize;
    fn get_current(&self) -> usize;
    fn get_max(&self) -> usize;
}