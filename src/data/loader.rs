use macroquad_toolkit::data_loader::load_embedded_json_labeled;
use serde::Deserialize;

use crate::data::{
    BuildingDefinition, EggTypeDefinition, EnemyDefinition, GameConfig, GameData,
    MonsterSpeciesDefinition, NpcDefinition, ResourceDefinition, TowerEventDefinition,
    TowerFloorDefinition, TowerHazardDefinition, TowerSpecialLocationDefinition,
};

#[derive(Debug, Deserialize)]
struct ConfigFile {
    config: GameConfig,
}

#[derive(Debug, Deserialize)]
struct ResourcesFile {
    resources: Vec<ResourceDefinition>,
}

#[derive(Debug, Deserialize)]
struct BuildingsFile {
    buildings: Vec<BuildingDefinition>,
}

#[derive(Debug, Deserialize)]
struct MonsterSpeciesFile {
    monster_species: Vec<MonsterSpeciesDefinition>,
}

#[derive(Debug, Deserialize)]
struct EggTypesFile {
    egg_types: Vec<EggTypeDefinition>,
}

#[derive(Debug, Deserialize)]
struct TowerFloorsFile {
    tower_floors: Vec<TowerFloorDefinition>,
}

#[derive(Debug, Deserialize)]
struct EnemiesFile {
    enemies: Vec<EnemyDefinition>,
}

#[derive(Debug, Deserialize)]
struct TowerSpecialsFile {
    special_locations: Vec<TowerSpecialLocationDefinition>,
    events: Vec<TowerEventDefinition>,
}

#[derive(Debug, Deserialize)]
struct TowerHazardsFile {
    hazards: Vec<TowerHazardDefinition>,
}

#[derive(Debug, Deserialize)]
struct NpcsFile {
    npcs: Vec<NpcDefinition>,
}

#[derive(Debug, Deserialize)]
struct BalanceFile {
    balance: crate::data::BalanceData,
}

pub struct GameDataLoader;

impl GameDataLoader {
    pub fn load_embedded() -> Result<GameData, String> {
        let config: ConfigFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/config.json"),
            "config",
        )?;
        let resources: ResourcesFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/resources.json"),
            "resources",
        )?;
        let buildings: BuildingsFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/buildings.json"),
            "buildings",
        )?;
        let species: MonsterSpeciesFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/monster_species.json"),
            "monster species",
        )?;
        let eggs: EggTypesFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/egg_types.json"),
            "egg types",
        )?;
        let tower_floors: TowerFloorsFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/tower_floors.json"),
            "tower floors",
        )?;
        let enemies: EnemiesFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/enemies.json"),
            "enemies",
        )?;
        let tower_specials: TowerSpecialsFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/tower_specials.json"),
            "tower specials",
        )?;
        let tower_hazards: TowerHazardsFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/tower_hazards.json"),
            "tower hazards",
        )?;
        let npcs: NpcsFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/npcs.json"),
            "npcs",
        )?;
        let balance: BalanceFile = parse_json(
            macroquad_toolkit::include_json_str!("../../assets/data/balance.json"),
            "balance",
        )?;

        GameData::from_parts(
            config.config,
            balance.balance,
            resources.resources,
            buildings.buildings,
            species.monster_species,
            eggs.egg_types,
            tower_floors.tower_floors,
            enemies.enemies,
            tower_specials.special_locations,
            tower_specials.events,
            tower_hazards.hazards,
            npcs.npcs,
        )
    }
}

fn parse_json<T>(json: &str, label: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    load_embedded_json_labeled(label, json)
}
