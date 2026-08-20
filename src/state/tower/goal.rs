//! Expedition goals and their player-facing descriptions.

use std::fmt;

use serde::{Deserialize, Serialize};

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

pub fn survey_charges_for(goal: TowerRunGoal) -> u32 {
    if goal == TowerRunGoal::Scout {
        3
    } else {
        2
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
            Self::EggHunt => "More egg rooms and nests; material caches are smaller.",
            Self::Salvage => "More wood, stone, ore, and coins; egg finds are rarer.",
            Self::Scout => "Fewer enemies and more open routes; rewards are modest.",
            Self::PushDeeper => "More stairs and deeper routes; enemies are denser.",
            Self::SafeRun => "Fewer enemies and traps; fewer rewards.",
        }
    }

    pub fn contract_id(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::EggHunt => "egg_hunt",
            Self::Salvage => "salvage",
            Self::Scout => "scout",
            Self::PushDeeper => "push_deeper",
            Self::SafeRun => "safe_run",
        }
    }
}

impl fmt::Display for TowerRunGoal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}
