use crate::attribute::attribute;
use crate::attribute::attribute::{AttributeContainer, JsonType};
use crate::state::position::Position;

pub trait Element : JsonType {
    fn position(&self) -> &Position;
    fn set_position(&mut self, position: Position);
}

impl Element for AttributeContainer {
    fn position(&self) -> &Position {
        self.get_unsafe(&attribute::POSITION)
    }
    fn set_position(&mut self, position: Position) {
        self.put(&attribute::POSITION, position);
    }
}

