use serde::{Deserialize, Serialize};

use crate::state::ResourceStack;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerProgress {
    pub best_floor: u32,
    pub unlocked_floor: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerRunState {
    pub current_floor: u32,
    pub rooms_explored: u32,
    pub pressure: u32,
    pub pressure_limit: u32,
    pub cargo: Vec<ResourceStack>,
    pub found_eggs: Vec<TowerFoundEgg>,
    pub event_log: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerFoundEgg {
    pub egg_type_id: String,
    pub hatch_days: u32,
    pub origin_floor: u32,
    pub palette_seed: u64,
}

impl TowerRunState {
    pub fn new(current_floor: u32, pressure_limit: u32) -> Self {
        Self {
            current_floor,
            rooms_explored: 0,
            pressure: 0,
            pressure_limit,
            cargo: Vec::new(),
            found_eggs: Vec::new(),
            event_log: vec![format!("Entered floor {current_floor}.")],
        }
    }

    pub fn add_cargo(&mut self, resource_id: &str, amount: i32) {
        if let Some(stack) = self
            .cargo
            .iter_mut()
            .find(|stack| stack.resource_id == resource_id)
        {
            stack.amount += amount;
            return;
        }

        self.cargo.push(ResourceStack {
            resource_id: resource_id.to_owned(),
            amount,
        });
    }

    pub fn cargo_amount(&self) -> i32 {
        self.cargo.iter().map(|stack| stack.amount.max(0)).sum()
    }

    pub fn add_event(&mut self, message: String) {
        self.event_log.push(message);
        if self.event_log.len() > 7 {
            let overflow = self.event_log.len() - 7;
            self.event_log.drain(0..overflow);
        }
    }
}
