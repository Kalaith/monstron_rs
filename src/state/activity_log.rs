use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActivityLog {
    pub next_id: u64,
    pub entries: Vec<LogEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub id: u64,
    pub day: u32,
    pub message: String,
}

impl ActivityLog {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, day: u32, message: String) {
        self.entries.push(LogEntry {
            id: self.next_id,
            day,
            message,
        });
        self.next_id += 1;

        if self.entries.len() > 80 {
            let overflow = self.entries.len() - 80;
            self.entries.drain(0..overflow);
        }
    }
}
