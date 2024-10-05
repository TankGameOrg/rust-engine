use crate::state::meta::player::PlayerRef;
use crate::state::position::Position;
use crate::util::attribute::JsonType;
use serde::{Deserialize, Serialize};

pub type ProposalRef = u32;

#[derive(Deserialize, Serialize, Debug)]
pub struct Proposal {
    proposal: ProposalType,
    reference: ProposalRef,
    votes: u32,
}

impl Proposal {
    pub fn new(proposal: ProposalType, reference: ProposalRef) -> Proposal {
        Proposal {
            proposal,
            reference,
            votes: 1u32,
        }
    }
}

#[typetag::serde]
impl JsonType for Proposal {}

#[typetag::serde]
impl JsonType for Vec<Proposal> {}

#[derive(Serialize, Deserialize, Debug)]
pub enum ProposalType {
    RepairBridge(Position),
    Bounty(PlayerRef),
    Expose(PlayerRef),
    SpawnWall(Position),
    DeclareHoliday,
    NuclearDisarmament,
}

#[typetag::serde]
impl JsonType for ProposalType {}
