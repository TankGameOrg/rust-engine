use crate::attribute::attribute;
use crate::attribute::attribute::AttributeContainer;
use crate::state::position::Position;
use as_any::AsAny;
use std::fmt::Debug;

#[typetag::serde]
pub trait Element: AsAny + Debug {
    fn position(&self) -> &Position;
    fn set_position(&mut self, position: Position);
}

#[typetag::serde]
pub trait Unit: Element {}

#[typetag::serde]
pub trait Floor: Element {
    fn is_walkable(&self) -> bool;
}

#[typetag::serde]
impl Element for AttributeContainer {
    fn position(&self) -> &Position {
        self.get_unsafe(&attribute::POSITION)
    }
    fn set_position(&mut self, position: Position) {
        self.put(&attribute::POSITION, position);
    }
}

#[typetag::serde]
impl Unit for AttributeContainer {}

#[typetag::serde]
impl Floor for AttributeContainer {
    fn is_walkable(&self) -> bool {
        todo!()
    }
}
