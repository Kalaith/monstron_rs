use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameConfig {
    pub save_version: u32,
    pub starting_day: u32,
    pub starter_species_id: String,
    pub starter_name: String,
    pub starting_log: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceDefinition {
    pub id: String,
    pub name: String,
    pub starting_amount: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceAmount {
    pub resource_id: String,
    pub amount: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildingDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub starting_level: u32,
    pub max_level: u32,
    pub upgrade_cost: Vec<ResourceAmount>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Element {
    Water,
    Fire,
    Earth,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Temperament {
    Loyal,
    Patient,
    Curious,
    Brave,
    Restless,
    Gentle,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MonsterRole {
    Scout,
    Tank,
    Support,
    Striker,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PassiveSkill {
    #[serde(rename = "Finds small loot")]
    FindsSmallLoot,
    #[serde(rename = "Resists poison")]
    ResistsPoison,
    #[serde(rename = "Detects eggs")]
    DetectsEggs,
    #[serde(rename = "Finds stone")]
    FindsStone,
    #[serde(rename = "Burns brambles")]
    BurnsBrambles,
    #[serde(rename = "Soothes injuries")]
    SoothesInjuries,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TownSkill {
    #[serde(rename = "Hatchery helper")]
    HatcheryHelper,
    Farming,
    Lighting,
    Guarding,
    #[serde(rename = "Workshop heat")]
    WorkshopHeat,
    Hatching,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MonsterSpeciesDefinition {
    pub id: String,
    pub name: String,
    pub element: Element,
    pub temperament: Temperament,
    pub role: MonsterRole,
    pub passive: PassiveSkill,
    pub town_skill: TownSkill,
    pub base_hp: i32,
    pub base_attack: i32,
    pub base_defense: i32,
    pub base_speed: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EggTypeDefinition {
    pub id: String,
    pub name: String,
    pub rarity: String,
    pub hatch_days: u32,
    pub discovery_floor: u32,
    pub possible_species: Vec<String>,
    pub element_bias: Vec<Element>,
    #[serde(default)]
    pub temperament_bias: Vec<Temperament>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerFloorDefinition {
    pub floor: u32,
    pub name: String,
    pub theme: String,
    pub enemy_hint: String,
    pub loot: Vec<ResourceAmount>,
    pub egg_types: Vec<String>,
    pub pressure_limit: u32,
    pub unlocks_floor: u32,
    #[serde(default)]
    pub is_boss_floor: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NpcDefinition {
    pub id: String,
    pub name: String,
    pub service: String,
    pub description: String,
}

impl fmt::Display for Element {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Water => "Water",
            Self::Fire => "Fire",
            Self::Earth => "Earth",
        })
    }
}

impl fmt::Display for Temperament {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Loyal => "Loyal",
            Self::Patient => "Patient",
            Self::Curious => "Curious",
            Self::Brave => "Brave",
            Self::Restless => "Restless",
            Self::Gentle => "Gentle",
        })
    }
}

impl fmt::Display for MonsterRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Scout => "Scout",
            Self::Tank => "Tank",
            Self::Support => "Support",
            Self::Striker => "Striker",
        })
    }
}

impl fmt::Display for PassiveSkill {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::FindsSmallLoot => "Finds small loot",
            Self::ResistsPoison => "Resists poison",
            Self::DetectsEggs => "Detects eggs",
            Self::FindsStone => "Finds stone",
            Self::BurnsBrambles => "Burns brambles",
            Self::SoothesInjuries => "Soothes injuries",
        })
    }
}

impl fmt::Display for TownSkill {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::HatcheryHelper => "Hatchery helper",
            Self::Farming => "Farming",
            Self::Lighting => "Lighting",
            Self::Guarding => "Guarding",
            Self::WorkshopHeat => "Workshop heat",
            Self::Hatching => "Hatching",
        })
    }
}
