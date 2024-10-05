use crate::rule::log_entry::LogEntry;
use crate::state::meta::player::PlayerRef;
use crate::state::state::State;

pub struct Context<'a> {
    pub state: &'a mut State,
    pub player: PlayerRef,
    pub log_entry: LogEntry,
}

impl<'a> Context<'a> {
    pub fn new(state: &'a mut State, player: PlayerRef, log_entry: LogEntry) -> Context<'a> {
        Context {
            state,
            player,
            log_entry,
        }
    }
}
