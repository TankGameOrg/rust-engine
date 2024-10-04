use crate::util::attribute::{AttributeContainer, Container, JsonType};
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use crate::util::attributes::NAME;

#[typetag::serde]
pub trait Player: Container + Debug {
    fn name(&self) -> &String;
}

#[typetag::serde]
impl Player for AttributeContainer {
    fn name(&self) -> &String {
        self.get_unsafe(&NAME)
    }
}

pub fn new_player() -> Box<dyn Player> {
    Box::new(AttributeContainer::new())
}

pub type Players = Vec<Box<dyn Player>>;

pub fn new_players() -> Players {
    Vec::new()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerRef {
    player: String,
}

impl PlayerRef {
    pub fn new(player: String) -> Self {
        PlayerRef { player }
    }

    pub fn name(&self) -> &String {
        &self.player
    }
}

#[typetag::serde]
impl JsonType for PlayerRef {}
