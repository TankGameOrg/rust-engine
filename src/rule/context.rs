use crate::rule::log_entry::LogEntry;
use crate::state::state::State;

pub struct Context<'a> {
    pub state: &'a mut State,
    pub log_entry: LogEntry,
}

impl<'a> Context<'a> {
    pub fn new(state: &'a mut State, log_entry: LogEntry) -> Context<'a> {
        Context { state, log_entry }
    }
}
