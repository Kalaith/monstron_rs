mod game_data;
mod loader;
mod schema;

pub use game_data::GameData;
pub use loader::GameDataLoader;
pub use schema::{
    BuildingDefinition, EggTypeDefinition, Element, GameConfig, MonsterRole,
    MonsterSpeciesDefinition, NpcDefinition, PassiveSkill, ResourceAmount, ResourceDefinition,
    Temperament, TowerFloorDefinition, TownSkill,
};
