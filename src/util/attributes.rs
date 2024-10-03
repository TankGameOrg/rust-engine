use crate::state::position::Position;
use crate::util::attribute::Attribute;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref POSITION: Attribute<Position> = Attribute::new("POSITION");
}
