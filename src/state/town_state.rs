use serde::{Deserialize, Serialize};

use crate::data::GameData;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TownState {
    pub buildings: Vec<BuildingState>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildingState {
    pub building_id: String,
    pub level: u32,
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
}
