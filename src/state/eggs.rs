use serde::{Deserialize, Serialize};

use std::fmt;

use crate::data::{Element, PassiveSkill, Temperament};
use crate::state::MonsterArtProfile;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EggInventory {
    pub next_id: u64,
    pub eggs: Vec<EggInstance>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EggInheritance {
    pub parent_ids: Vec<u64>,
    pub species_options: Vec<String>,
    pub element_options: Vec<Element>,
    pub temperament_options: Vec<Temperament>,
    pub passive_options: Vec<PassiveSkill>,
    pub mutation_floor: u32,
    pub mutated: bool,
    #[serde(default)]
    pub lineage_quality: u32,
    #[serde(default)]
    pub art_profile: MonsterArtProfile,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EggInstance {
    pub id: u64,
    pub egg_type_id: String,
    pub days_remaining: u32,
    pub origin_floor: u32,
    pub palette_seed: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inheritance: Option<EggInheritance>,
    #[serde(default)]
    pub care_focus: EggCareFocus,
    #[serde(default)]
    pub care_points: u32,
    #[serde(default)]
    pub studied: bool,
    #[serde(default)]
    pub stabilised: bool,
    #[serde(default)]
    pub last_care_day: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum EggCareFocus {
    #[default]
    None,
    Warm,
    Soothe,
    Study,
    Stabilise,
}

impl EggInventory {
    pub fn add_egg(
        &mut self,
        egg_type_id: String,
        days_remaining: u32,
        origin_floor: u32,
        palette_seed: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.eggs.push(EggInstance {
            id,
            egg_type_id,
            days_remaining,
            origin_floor,
            palette_seed,
            inheritance: None,
            care_focus: EggCareFocus::None,
            care_points: 0,
            studied: false,
            stabilised: false,
            last_care_day: 0,
        });
        id
    }

    pub fn add_bred_egg(
        &mut self,
        egg_type_id: String,
        days_remaining: u32,
        origin_floor: u32,
        palette_seed: u64,
        inheritance: EggInheritance,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.eggs.push(EggInstance {
            id,
            egg_type_id,
            days_remaining,
            origin_floor,
            palette_seed,
            inheritance: Some(inheritance),
            care_focus: EggCareFocus::None,
            care_points: 0,
            studied: false,
            stabilised: false,
            last_care_day: 0,
        });
        id
    }

    pub fn egg_mut(&mut self, egg_id: u64) -> Option<&mut EggInstance> {
        self.eggs.iter_mut().find(|egg| egg.id == egg_id)
    }

    pub fn remove_egg(&mut self, egg_id: u64) -> Option<EggInstance> {
        let index = self.eggs.iter().position(|egg| egg.id == egg_id)?;
        Some(self.eggs.remove(index))
    }
}

impl fmt::Display for EggCareFocus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::None => "Uncared",
            Self::Warm => "Warmed",
            Self::Soothe => "Soothed",
            Self::Study => "Studied",
            Self::Stabilise => "Stabilised",
        })
    }
}
