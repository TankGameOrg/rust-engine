use crate::state::board::Board;
use crate::state::meta::meta::Meta;
use crate::state::meta::player::{Player, PlayerRef, Players};
use crate::util::attribute::JsonType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    board: Board,
    players: Players,
    meta: Box<dyn Meta>,
}

impl State {
    pub fn new(board: Board, players: Players, meta: Box<dyn Meta>) -> Self {
        State {
            board,
            players,
            meta,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn players(&self) -> &Players {
        &self.players
    }

    pub fn meta(&self) -> &Box<dyn Meta> {
        &self.meta
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn players_mut(&mut self) -> &mut Players {
        &mut self.players
    }

    pub fn meta_mut(&mut self) -> &mut Box<dyn Meta> {
        &mut self.meta
    }

    pub fn player_from_ref<'a>(&'a self, player_ref: &PlayerRef) -> &'a Box<dyn Player> {
        self.players
            .iter()
            .find(|p| p.name() == player_ref.name())
            .expect(format!("Player {} not found", player_ref.name()).as_str())
    }

    pub fn player_from_ref_mut<'a>(
        &'a mut self,
        player_ref: &PlayerRef,
    ) -> &'a mut Box<dyn Player> {
        self.players
            .iter_mut()
            .find(|p| p.name() == player_ref.name())
            .expect(format!("Player {} not found", player_ref.name()).as_str())
    }
}

#[typetag::serde]
impl JsonType for State {}
