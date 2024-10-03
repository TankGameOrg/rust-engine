use serde::{Deserialize, Serialize};
use crate::attribute::attribute::JsonType;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position { x, y }
    }
}

#[typetag::serde]
impl JsonType for Position {}
