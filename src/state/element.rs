use crate::state::meta::player::PlayerRef;
use crate::state::position::Position;
use crate::util::attribute::{AttributeContainer, Container};
use crate::util::attributes::{DURABILITY, PLAYER_REF, POSITION, WALKABLE};
use std::fmt::Debug;

#[typetag::serde]
pub trait Element: Container + Debug {
    fn position(&self) -> &Position;
    fn get_class(&self) -> &String;
    fn is_walkable(&self) -> bool;
}

#[typetag::serde]
impl Element for AttributeContainer {
    fn position(&self) -> &Position {
        self.get_unsafe(&POSITION)
    }
    fn get_class(&self) -> &String {
        self.get_class().expect("Element did not have a class")
    }
    fn is_walkable(&self) -> bool {
        if is_floor(
            self.get_class()
                .expect("Element did not have attribute class"),
        ) {
            match self.get(&WALKABLE) {
                Some(value) => *value,
                None => true,
            }
        } else {
            false
        }
    }
}

pub const TANK: &str = "Tank";
pub const WALL: &str = "Wall";
pub const UNBREAKABLE_WALL: &str = "UnbreakableWall";

pub const GOLD_MINE: &str = "GoldMine";
pub const CHASM: &str = "Chasm";
pub const BRIDGE: &str = "Bridge";


// The following functions expect the output element to be placed directly onto the board.
// They do not have a set position, this their position needs to be set if they are going to be used
// without being placed on a board.

pub fn new_tank(player_ref: PlayerRef) -> Box<dyn Element> {
    let mut container = AttributeContainer::new_with_class(TANK.to_string());
    container.put(&PLAYER_REF, player_ref);
    Box::new(container)
}

pub fn new_wall(durability: i32) -> Box<dyn Element> {
    let mut container = AttributeContainer::new_with_class(WALL.to_string());
    container.put(&DURABILITY, durability);
    Box::new(container)
}

pub fn new_unbreakable_wall() -> Box<dyn Element> {
    let container = AttributeContainer::new_with_class(UNBREAKABLE_WALL.to_string());
    Box::new(container)
}

pub fn new_gold_mine() -> Box<dyn Element> {
    let container = AttributeContainer::new_with_class(GOLD_MINE.to_string());
    Box::new(container)
}

pub fn new_chasm() -> Box<dyn Element> {
    let mut container = AttributeContainer::new_with_class(CHASM.to_string());
    container.put(&WALKABLE, false);
    Box::new(container)
}

pub fn new_bridge(position: Position, durability: i32) -> Box<dyn Element> {
    let mut container = AttributeContainer::new_with_class(BRIDGE.to_string());
    container.put(&POSITION, position);
    container.put(&DURABILITY, durability);
    container.put(&WALKABLE, true);
    Box::new(container)
}

pub fn is_unit(class: &String) -> bool {
    match class.as_str() {
        TANK | WALL | UNBREAKABLE_WALL => true,
        _ => false,
    }
}

pub fn is_floor(class: &String) -> bool {
    match class.as_str() {
        GOLD_MINE | CHASM | BRIDGE => true,
        _ => false,
    }
}
