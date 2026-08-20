//! Validated, indexed game content shared by engines and screens.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::data::{
    BalanceData, BuildingDefinition, EggTypeDefinition, EnemyDefinition, GameConfig,
    MonsterSpeciesDefinition, NpcDefinition, ResourceDefinition, ShopTradeDefinition,
    TowerAnomalyDefinition, TowerContractDefinition, TowerEventDefinition, TowerFloorDefinition,
    TowerHazardDefinition, TowerRewardDefinition, TowerSpecialLocationDefinition,
};

mod fallback;
mod indexes;
mod validation;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameData {
    pub config: GameConfig,
    pub balance: BalanceData,
    pub resources: Vec<ResourceDefinition>,
    pub buildings: Vec<BuildingDefinition>,
    pub monster_species: Vec<MonsterSpeciesDefinition>,
    pub egg_types: Vec<EggTypeDefinition>,
    pub tower_floors: Vec<TowerFloorDefinition>,
    pub enemies: Vec<EnemyDefinition>,
    pub tower_special_locations: Vec<TowerSpecialLocationDefinition>,
    pub tower_events: Vec<TowerEventDefinition>,
    pub tower_hazards: Vec<TowerHazardDefinition>,
    pub tower_contracts: Vec<TowerContractDefinition>,
    pub tower_anomalies: Vec<TowerAnomalyDefinition>,
    pub npcs: Vec<NpcDefinition>,
    #[serde(skip)]
    resource_index: HashMap<String, usize>,
    #[serde(skip)]
    building_index: HashMap<String, usize>,
    #[serde(skip)]
    species_index: HashMap<String, usize>,
    #[serde(skip)]
    egg_index: HashMap<String, usize>,
    #[serde(skip)]
    tower_floor_index: HashMap<u32, usize>,
    #[serde(skip)]
    enemy_index: HashMap<String, usize>,
    #[serde(skip)]
    tower_special_location_index: HashMap<String, usize>,
    #[serde(skip)]
    tower_event_index: HashMap<String, usize>,
    #[serde(skip)]
    tower_hazard_index: HashMap<String, usize>,
    #[serde(skip)]
    tower_contract_index: HashMap<String, usize>,
    #[serde(skip)]
    tower_anomaly_index: HashMap<String, usize>,
    #[serde(skip)]
    npc_index: HashMap<String, usize>,
    #[serde(skip)]
    stat_curve_index: HashMap<String, usize>,
    #[serde(skip)]
    cooldown_index: HashMap<String, usize>,
    #[serde(skip)]
    shop_trade_index: HashMap<String, usize>,
    #[serde(skip)]
    tower_reward_index: HashMap<u32, usize>,
}

impl GameData {
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        config: GameConfig,
        balance: BalanceData,
        resources: Vec<ResourceDefinition>,
        buildings: Vec<BuildingDefinition>,
        monster_species: Vec<MonsterSpeciesDefinition>,
        egg_types: Vec<EggTypeDefinition>,
        tower_floors: Vec<TowerFloorDefinition>,
        enemies: Vec<EnemyDefinition>,
        tower_special_locations: Vec<TowerSpecialLocationDefinition>,
        tower_events: Vec<TowerEventDefinition>,
        tower_hazards: Vec<TowerHazardDefinition>,
        tower_contracts: Vec<TowerContractDefinition>,
        tower_anomalies: Vec<TowerAnomalyDefinition>,
        npcs: Vec<NpcDefinition>,
    ) -> Result<Self, String> {
        let mut data = Self {
            config,
            balance,
            resources,
            buildings,
            monster_species,
            egg_types,
            tower_floors,
            enemies,
            tower_special_locations,
            tower_events,
            tower_hazards,
            tower_contracts,
            tower_anomalies,
            npcs,
            resource_index: HashMap::new(),
            building_index: HashMap::new(),
            species_index: HashMap::new(),
            egg_index: HashMap::new(),
            tower_floor_index: HashMap::new(),
            enemy_index: HashMap::new(),
            tower_special_location_index: HashMap::new(),
            tower_event_index: HashMap::new(),
            tower_hazard_index: HashMap::new(),
            tower_contract_index: HashMap::new(),
            tower_anomaly_index: HashMap::new(),
            npc_index: HashMap::new(),
            stat_curve_index: HashMap::new(),
            cooldown_index: HashMap::new(),
            shop_trade_index: HashMap::new(),
            tower_reward_index: HashMap::new(),
        };
        indexes::build(&mut data)?;
        validation::validate(&data)?;
        Ok(data)
    }

    pub fn fallback() -> Self {
        fallback::build()
    }

    pub fn building(&self, id: &str) -> Option<&BuildingDefinition> {
        self.building_index
            .get(id)
            .and_then(|index| self.buildings.get(*index))
    }

    pub fn resource_name<'a>(&'a self, id: &'a str) -> &'a str {
        self.resource_index
            .get(id)
            .and_then(|index| self.resources.get(*index))
            .map(|resource| resource.name.as_str())
            .unwrap_or(id)
    }

    pub fn species(&self, id: &str) -> Option<&MonsterSpeciesDefinition> {
        self.species_index
            .get(id)
            .and_then(|index| self.monster_species.get(*index))
    }

    pub fn egg_type(&self, id: &str) -> Option<&EggTypeDefinition> {
        self.egg_index
            .get(id)
            .and_then(|index| self.egg_types.get(*index))
    }

    pub fn tower_floor(&self, floor: u32) -> Option<&TowerFloorDefinition> {
        self.tower_floor_index
            .get(&floor)
            .and_then(|index| self.tower_floors.get(*index))
    }

    pub fn enemy(&self, id: &str) -> Option<&EnemyDefinition> {
        self.enemy_index
            .get(id)
            .and_then(|index| self.enemies.get(*index))
    }

    pub fn tower_special_location(&self, id: &str) -> Option<&TowerSpecialLocationDefinition> {
        self.tower_special_location_index
            .get(id)
            .and_then(|index| self.tower_special_locations.get(*index))
    }

    pub fn tower_event(&self, id: &str) -> Option<&TowerEventDefinition> {
        self.tower_event_index
            .get(id)
            .and_then(|index| self.tower_events.get(*index))
    }

    pub fn tower_hazard(&self, id: &str) -> Option<&TowerHazardDefinition> {
        self.tower_hazard_index
            .get(id)
            .and_then(|index| self.tower_hazards.get(*index))
    }

    pub fn tower_contract(&self, id: &str) -> Option<&TowerContractDefinition> {
        self.tower_contract_index
            .get(id)
            .and_then(|index| self.tower_contracts.get(*index))
    }

    pub fn tower_anomaly(&self, id: &str) -> Option<&TowerAnomalyDefinition> {
        self.tower_anomaly_index
            .get(id)
            .and_then(|index| self.tower_anomalies.get(*index))
    }

    pub fn npc(&self, id: &str) -> Option<&NpcDefinition> {
        self.npc_index
            .get(id)
            .and_then(|index| self.npcs.get(*index))
    }

    pub fn stat_curve(&self, species_id: &str) -> Option<&crate::data::MonsterStatCurveDefinition> {
        self.stat_curve_index
            .get(species_id)
            .and_then(|index| self.balance.monster_stat_curves.get(*index))
    }

    pub fn combat_cooldown(&self, id: &str) -> Option<u32> {
        self.cooldown_index
            .get(id)
            .and_then(|index| self.balance.combat_cooldowns.get(*index))
            .map(|definition| definition.turns)
    }

    pub fn shop_trade(&self, id: &str) -> Option<&ShopTradeDefinition> {
        self.shop_trade_index
            .get(id)
            .and_then(|index| self.balance.shop_trades.get(*index))
    }

    pub fn tower_reward(&self, floor: u32) -> Option<&TowerRewardDefinition> {
        self.tower_reward_index
            .get(&floor)
            .and_then(|index| self.balance.tower_rewards.get(*index))
    }
}
