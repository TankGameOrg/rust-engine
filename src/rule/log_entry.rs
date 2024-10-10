use crate::util::attribute::AttributeContainer;

pub enum EntryData {
    StartOfDay,
    PlayerAction(String),
}

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
