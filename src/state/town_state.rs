use serde::{Deserialize, Serialize};

use crate::data::GameData;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TownState {
    pub buildings: Vec<BuildingState>,
    #[serde(default)]
    pub assignments: Vec<TownAssignment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildingState {
    pub building_id: String,
    pub level: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TownAssignment {
    pub monster_id: u64,
    pub job: TownJobKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TownJobKind {
    Forage,
    Quarry,
    Workshop,
    HatcheryCare,
}

impl TownState {
    pub fn from_data(data: &GameData) -> Self {
        Self {
            buildings: data
                .buildings
                .iter()
                .filter(|building| building.starting_level > 0)
                .map(|building| BuildingState {
                    building_id: building.id.clone(),
                    level: building.starting_level,
                })
                .collect(),
            assignments: Vec::new(),
        }
    }

    pub fn building_level(&self, building_id: &str) -> u32 {
        self.buildings
            .iter()
            .find(|building| building.building_id == building_id)
            .map(|building| building.level)
            .unwrap_or(0)
    }

    pub fn set_building_level(&mut self, building_id: &str, level: u32) {
        if let Some(building) = self
            .buildings
            .iter_mut()
            .find(|building| building.building_id == building_id)
        {
            building.level = level;
            return;
        }

        self.buildings.push(BuildingState {
            building_id: building_id.to_owned(),
            level,
        });
    }

    pub fn monster_job(&self, monster_id: u64) -> Option<TownJobKind> {
        self.assignments
            .iter()
            .find(|assignment| assignment.monster_id == monster_id)
            .map(|assignment| assignment.job)
    }

    pub fn set_monster_job(&mut self, monster_id: u64, job: TownJobKind) {
        if let Some(assignment) = self
            .assignments
            .iter_mut()
            .find(|assignment| assignment.monster_id == monster_id)
        {
            assignment.job = job;
            return;
        }

        self.assignments.push(TownAssignment { monster_id, job });
    }

    pub fn clear_monster_job(&mut self, monster_id: u64) -> bool {
        let Some(index) = self
            .assignments
            .iter()
            .position(|assignment| assignment.monster_id == monster_id)
        else {
            return false;
        };
        self.assignments.remove(index);
        true
    }
}
