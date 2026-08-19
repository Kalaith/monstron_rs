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
pub struct MonsterStatCurveDefinition {
    pub species_id: String,
    pub hp_per_level: i32,
    pub attack_per_level: i32,
    pub defense_per_level: i32,
    pub speed_per_interval: i32,
    pub speed_interval: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CombatCooldownDefinition {
    pub id: String,
    pub turns: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShopTradeDefinition {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub cost: Vec<ResourceAmount>,
    pub reward: Vec<ResourceAmount>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerRewardDefinition {
    pub floor: u32,
    pub rewards: Vec<ResourceAmount>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BalanceData {
    pub monster_stat_curves: Vec<MonsterStatCurveDefinition>,
    pub combat_cooldowns: Vec<CombatCooldownDefinition>,
    pub shop_trades: Vec<ShopTradeDefinition>,
    pub tower_rewards: Vec<TowerRewardDefinition>,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerLocationVisual {
    Shrine,
    RecoverySpring,
    AncientMachinery,
    RelicArchive,
    SecretClue,
    Hazard,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerSpecialLocationDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub min_floor: u32,
    pub max_floor: u32,
    pub visual: TowerLocationVisual,
    pub event_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerEventDefinition {
    pub id: String,
    pub name: String,
    pub narrative: String,
    #[serde(default)]
    pub rewards: Vec<ResourceAmount>,
    #[serde(default)]
    pub cargo_costs: Vec<ResourceAmount>,
    #[serde(default)]
    pub pressure_delta: i32,
    #[serde(default)]
    pub party_healing: i32,
    #[serde(default)]
    pub enemy_id: String,
    #[serde(default)]
    pub egg_type_id: String,
    #[serde(default)]
    pub reveal_map: bool,
    #[serde(default)]
    pub refresh_camp: bool,
    #[serde(default)]
    pub blessing: Option<TowerBlessing>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerBlessing {
    QuietSteps,
    Wardstone,
    CacheSense,
}

impl TowerBlessing {
    pub fn label(self) -> &'static str {
        match self {
            Self::QuietSteps => "Quiet Steps",
            Self::Wardstone => "Wardstone",
            Self::CacheSense => "Cache Sense",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerHazardVisual {
    Spores,
    Brambles,
    CinderVent,
    Flood,
    Frostfall,
    CrownPollen,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerHazardDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub min_floor: u32,
    pub max_floor: u32,
    pub visual: TowerHazardVisual,
    pub damage: i32,
    pub pressure: u32,
    #[serde(default)]
    pub counter_passive: Option<PassiveSkill>,
    #[serde(default)]
    pub counter_element: Option<Element>,
    #[serde(default)]
    pub counter_rewards: Vec<ResourceAmount>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerContractTarget {
    Eggs,
    Cargo,
    Steps,
    Floors,
    Landmarks,
    HazardsCountered,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerContractDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target: TowerContractTarget,
    pub target_amount: u32,
    pub rewards: Vec<ResourceAmount>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerAnomalyEffect {
    QuietVeil,
    EchoingRain,
    CacheBloom,
    MendingLights,
    NestingPulse,
    HunterTracks,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerAnomalyDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub min_floor: u32,
    pub max_floor: u32,
    pub visual_index: usize,
    pub effect: TowerAnomalyEffect,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnemyDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub min_floor: u32,
    pub max_floor: u32,
    #[serde(default)]
    pub is_boss: bool,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub xp_reward: u32,
    pub rewards: Vec<ResourceAmount>,
    #[serde(default)]
    pub behavior: EnemyBehavior,
    #[serde(default)]
    pub visual: DungeonEnemyVisual,
    #[serde(default = "default_enemy_pack_size")]
    pub pack_size: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum EnemyBehavior {
    #[default]
    Standard,
    Bruiser,
    Bulwark,
    Harrier,
    Hexer,
    Swarm,
    Ambusher,
    Regenerator,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum DungeonEnemyVisual {
    #[default]
    Crawler,
    Winged,
    Rooted,
    Wisp,
    Aquatic,
    Armored,
}

fn default_enemy_pack_size() -> u32 {
    1
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
