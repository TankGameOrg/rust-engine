use crate::state::position::Position;
use crate::util::attribute::{Attribute, AttributeContainer};
use lazy_static::lazy_static;
use crate::state::meta::player::PlayerRef;

lazy_static! {

    // Players
    pub static ref NAME: Attribute<String> = Attribute::new("NAME");

    // Council
    pub static ref COUNCIL: Attribute<AttributeContainer> = Attribute::new("COUNCIL");
    pub static ref COUNCILORS: Attribute<Vec<PlayerRef>> = Attribute::new("COUNCILORS");
    pub static ref SENATORS: Attribute<Vec<PlayerRef>> = Attribute::new("SENATORS");
    pub static ref COFFER: Attribute<i32> = Attribute::new("COFFER");
    pub static ref CAN_BOUNTY: Attribute<bool> = Attribute::new("CAN_BOUNTY");

    // Elements
    pub static ref POSITION: Attribute<Position> = Attribute::new("POSITION");
    pub static ref WALKABLE: Attribute<bool> = Attribute::new("WALKABLE");
}
