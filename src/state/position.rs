use serde::{Deserialize, Serialize};
use crate::attribute::attribute::JsonType;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position { x, y }
    }
}

#[typetag::serde]
impl JsonType for Position {}
