use crate::state::meta::player::PlayerRef;
use crate::util::attribute::{AttributeContainer, JsonType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum EntryData {
    StartOfDay,
    PlayerAction(PlayerRef, String), // initiator, action name
}

#[typetag::serde]
impl JsonType for EntryData {}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub timestamp: u64,
    pub entry_type: EntryData,
    pub fields: AttributeContainer,
}

impl LogEntry {
    pub fn new(timestamp: u64, entry_type: EntryData, fields: AttributeContainer) -> LogEntry {
        LogEntry {
            timestamp,
            entry_type,
            fields,
        }
    }
}

#[typetag::serde]
impl JsonType for LogEntry {}
