use crate::state::position::Position;
use crate::util::attribute::AttributeContainer;
use crate::util::attributes::POSITION;
use as_any::AsAny;
use std::fmt::Debug;

#[typetag::serde]
pub trait Element: AsAny + Debug {
    fn position(&self) -> &Position;
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
        self.get_unsafe(&POSITION)
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
