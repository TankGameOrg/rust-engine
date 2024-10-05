use crate::rule::proposal::Proposal;
use crate::state::meta::player::PlayerRef;
use crate::state::position::Position;
use crate::state::upgrade::Upgrade;
use crate::util::attribute::{Attribute, AttributeContainer};
use lazy_static::lazy_static;

lazy_static! {

    // Players
    pub static ref NAME: Attribute<String> = Attribute::new("NAME");
    pub static ref PLAYER_REF: Attribute<PlayerRef> = Attribute::new("PLAYER_REF");
    pub static ref ACTIONS: Attribute<i32> = Attribute::new("ACTIONS");
    pub static ref MAX_ACTIONS: Attribute<i32> = Attribute::new("MAX_ACTIONS");
    pub static ref VOTES: Attribute<i32> = Attribute::new("VOTES");

    // Council
    pub static ref COUNCIL: Attribute<AttributeContainer> = Attribute::new("COUNCIL");
    pub static ref COUNCILORS: Attribute<Vec<PlayerRef>> = Attribute::new("COUNCILORS");
    pub static ref COFFER: Attribute<i32> = Attribute::new("COFFER");

    // Elements
    pub static ref POSITION: Attribute<Position> = Attribute::new("POSITION");
    pub static ref DURABILITY: Attribute<i32> = Attribute::new("DURABILITY");

    // Floors
    pub static ref WALKABLE: Attribute<bool> = Attribute::new("WALKABLE");

    // Units
    pub static ref GOLD: Attribute<i32> = Attribute::new("GOLD");
    pub static ref RANGE: Attribute<i32> = Attribute::new("RANGE");
    pub static ref BOUNTY: Attribute<i32> = Attribute::new("BOUNTY");
    pub static ref UPGRADES: Attribute<Vec<Upgrade>> = Attribute::new("BOUNTY");

    // Proposals
    pub static ref PROPOSALS: Attribute<Vec<Proposal>> = Attribute::new("PROPOSALS");
    pub static ref PROPOSAL_COUNT: Attribute<Vec<i32>> = Attribute::new("PROPOSAL_COUNT");

    // Log entry
    pub static ref TARGET_POSITION: Attribute<Position> = Attribute::new("TARGET_POSITION");
    pub static ref TARGET_PLAYER: Attribute<PlayerRef> = Attribute::new("TARGET_PLAYER");
    pub static ref DAMAGE: Attribute<i32> = Attribute::new("DAMAGE");
    pub static ref DONATION: Attribute<i32> = Attribute::new("DONATION");
}
