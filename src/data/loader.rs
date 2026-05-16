use serde::Deserialize;

use crate::data::{
    BuildingDefinition, EggTypeDefinition, GameConfig, GameData, MonsterSpeciesDefinition,
    NpcDefinition, ResourceDefinition, TowerFloorDefinition,
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
struct NpcsFile {
    npcs: Vec<NpcDefinition>,
}

pub struct GameDataLoader;

impl GameDataLoader {
    pub fn load_embedded() -> Result<GameData, String> {
        let config: ConfigFile =
            parse_json(include_str!("../../assets/data/config.json"), "config")?;
        let resources: ResourcesFile = parse_json(
            include_str!("../../assets/data/resources.json"),
            "resources",
        )?;
        let buildings: BuildingsFile = parse_json(
            include_str!("../../assets/data/buildings.json"),
            "buildings",
        )?;
        let species: MonsterSpeciesFile = parse_json(
            include_str!("../../assets/data/monster_species.json"),
            "monster species",
        )?;
        let eggs: EggTypesFile = parse_json(
            include_str!("../../assets/data/egg_types.json"),
            "egg types",
        )?;
        let tower_floors: TowerFloorsFile = parse_json(
            include_str!("../../assets/data/tower_floors.json"),
            "tower floors",
        )?;
        let npcs: NpcsFile = parse_json(include_str!("../../assets/data/npcs.json"), "npcs")?;

        GameData::from_parts(
            config.config,
            resources.resources,
            buildings.buildings,
            species.monster_species,
            eggs.egg_types,
            tower_floors.tower_floors,
            npcs.npcs,
        )
    }
}

fn parse_json<T>(json: &str, label: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(json).map_err(|error| format!("Failed to parse {label}: {error}"))
}
