use serde::{Deserialize, Serialize};
use std::fmt;

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
    #[serde(default)]
    pub goal: TowerRunGoal,
    pub cargo: Vec<ResourceStack>,
    pub found_eggs: Vec<TowerFoundEgg>,
    pub event_log: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerRunGoal {
    #[default]
    Balanced,
    EggHunt,
    Salvage,
    Scout,
    PushDeeper,
    SafeRun,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerFoundEgg {
    pub egg_type_id: String,
    pub hatch_days: u32,
    pub origin_floor: u32,
    pub palette_seed: u64,
}

impl TowerRunState {
    pub fn new(current_floor: u32, pressure_limit: u32, goal: TowerRunGoal) -> Self {
        Self {
            current_floor,
            rooms_explored: 0,
            pressure: 0,
            pressure_limit,
            goal,
            cargo: Vec::new(),
            found_eggs: Vec::new(),
            event_log: vec![format!("Entered floor {current_floor} on a {goal} run.")],
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

impl TowerRunGoal {
    pub const CHOICES: [Self; 5] = [
        Self::EggHunt,
        Self::Salvage,
        Self::Scout,
        Self::PushDeeper,
        Self::SafeRun,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Balanced => "Balanced",
            Self::EggHunt => "Egg Hunt",
            Self::Salvage => "Salvage",
            Self::Scout => "Scout",
            Self::PushDeeper => "Push",
            Self::SafeRun => "Safe Run",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Self::Balanced => "Normal eggs, loot, danger, and floor progress.",
            Self::EggHunt => "More nest events and egg hints; material caches are smaller.",
            Self::Salvage => "More wood, stone, ore, and coins; egg finds are rarer.",
            Self::Scout => "Fewer fights and more floor information; rewards are modest.",
            Self::PushDeeper => "More stair events and progress; pressure climbs faster.",
            Self::SafeRun => "Lower pressure and injury risk; fewer rewards.",
        }
    }
}

impl fmt::Display for TowerRunGoal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}
