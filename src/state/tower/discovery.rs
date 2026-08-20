//! Persistent Field Guide discoveries collected during tower runs.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TowerDiscoveryState {
    pub enemy_ids: Vec<String>,
    pub special_location_ids: Vec<String>,
    pub hazard_ids: Vec<String>,
    #[serde(default)]
    pub event_ids: Vec<String>,
}

impl TowerDiscoveryState {
    pub fn record_count(&self) -> usize {
        self.enemy_ids.len() + self.special_location_ids.len() + self.hazard_ids.len()
    }

    pub fn survey_bonus(&self) -> u32 {
        match self.record_count() {
            30.. => 2,
            12.. => 1,
            _ => 0,
        }
    }

    pub fn discover_enemy(&mut self, id: &str) -> bool {
        discover_id(&mut self.enemy_ids, id)
    }

    pub fn discover_special_location(&mut self, id: &str) -> bool {
        discover_id(&mut self.special_location_ids, id)
    }

    pub fn discover_hazard(&mut self, id: &str) -> bool {
        discover_id(&mut self.hazard_ids, id)
    }

    pub fn discover_event(&mut self, id: &str) -> bool {
        discover_id(&mut self.event_ids, id)
    }
}

fn discover_id(ids: &mut Vec<String>, id: &str) -> bool {
    if id.is_empty() || ids.iter().any(|known| known == id) {
        false
    } else {
        ids.push(id.to_owned());
        true
    }
}
