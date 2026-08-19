use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::data::{
    BalanceData, BuildingDefinition, EggTypeDefinition, Element, EnemyDefinition, GameConfig,
    MonsterRole, MonsterSpeciesDefinition, NpcDefinition, PassiveSkill, ResourceDefinition,
    ShopTradeDefinition, Temperament, TowerAnomalyDefinition, TowerContractDefinition,
    TowerEventDefinition, TowerFloorDefinition, TowerHazardDefinition, TowerRewardDefinition,
    TowerSpecialLocationDefinition, TownSkill,
};

mod fallback;

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
        data.build_indexes()?;
        data.validate_references()?;
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

    fn build_indexes(&mut self) -> Result<(), String> {
        self.resource_index = build_unique_index(
            self.resources
                .iter()
                .enumerate()
                .map(|(index, resource)| (&resource.id, index)),
            "resource",
        )?;
        self.building_index = build_unique_index(
            self.buildings
                .iter()
                .enumerate()
                .map(|(index, building)| (&building.id, index)),
            "building",
        )?;
        self.species_index = build_unique_index(
            self.monster_species
                .iter()
                .enumerate()
                .map(|(index, species)| (&species.id, index)),
            "monster species",
        )?;
        self.egg_index = build_unique_index(
            self.egg_types
                .iter()
                .enumerate()
                .map(|(index, egg)| (&egg.id, index)),
            "egg type",
        )?;
        self.tower_floor_index = build_unique_floor_index(
            self.tower_floors
                .iter()
                .enumerate()
                .map(|(index, floor)| (floor.floor, index)),
        )?;
        self.enemy_index = build_unique_index(
            self.enemies
                .iter()
                .enumerate()
                .map(|(index, enemy)| (&enemy.id, index)),
            "enemy",
        )?;
        self.tower_special_location_index = build_unique_index(
            self.tower_special_locations
                .iter()
                .enumerate()
                .map(|(index, location)| (&location.id, index)),
            "tower special location",
        )?;
        self.tower_event_index = build_unique_index(
            self.tower_events
                .iter()
                .enumerate()
                .map(|(index, event)| (&event.id, index)),
            "tower event",
        )?;
        self.tower_hazard_index = build_unique_index(
            self.tower_hazards
                .iter()
                .enumerate()
                .map(|(index, hazard)| (&hazard.id, index)),
            "tower hazard",
        )?;
        self.tower_contract_index = build_unique_index(
            self.tower_contracts
                .iter()
                .enumerate()
                .map(|(index, contract)| (&contract.id, index)),
            "tower contract",
        )?;
        self.tower_anomaly_index = build_unique_index(
            self.tower_anomalies
                .iter()
                .enumerate()
                .map(|(index, anomaly)| (&anomaly.id, index)),
            "tower anomaly",
        )?;
        self.npc_index = build_unique_index(
            self.npcs
                .iter()
                .enumerate()
                .map(|(index, npc)| (&npc.id, index)),
            "npc",
        )?;
        self.stat_curve_index = build_unique_index(
            self.balance
                .monster_stat_curves
                .iter()
                .enumerate()
                .map(|(index, curve)| (&curve.species_id, index)),
            "monster stat curve",
        )?;
        self.cooldown_index = build_unique_index(
            self.balance
                .combat_cooldowns
                .iter()
                .enumerate()
                .map(|(index, cooldown)| (&cooldown.id, index)),
            "combat cooldown",
        )?;
        self.shop_trade_index = build_unique_index(
            self.balance
                .shop_trades
                .iter()
                .enumerate()
                .map(|(index, trade)| (&trade.id, index)),
            "shop trade",
        )?;
        self.tower_reward_index = build_unique_floor_index(
            self.balance
                .tower_rewards
                .iter()
                .enumerate()
                .map(|(index, reward)| (reward.floor, index)),
        )?;
        Ok(())
    }

    fn validate_references(&self) -> Result<(), String> {
        if self.species(&self.config.starter_species_id).is_none() {
            return Err(format!(
                "Starter species '{}' does not exist",
                self.config.starter_species_id
            ));
        }

        let resource_ids = self.resource_ids();
        for building in &self.buildings {
            for cost in &building.upgrade_cost {
                if !resource_ids.contains(&cost.resource_id) {
                    return Err(format!(
                        "Building '{}' references missing resource '{}'",
                        building.id, cost.resource_id
                    ));
                }
            }
        }

        for egg in &self.egg_types {
            for species_id in &egg.possible_species {
                if self.species(species_id).is_none() {
                    return Err(format!(
                        "Egg '{}' references missing species '{}'",
                        egg.id, species_id
                    ));
                }
            }
        }

        for floor in &self.tower_floors {
            if floor.pressure_limit == 0 {
                return Err(format!(
                    "Tower floor {} has a zero pressure limit",
                    floor.floor
                ));
            }

            for loot in &floor.loot {
                if !resource_ids.contains(&loot.resource_id) {
                    return Err(format!(
                        "Tower floor {} references missing resource '{}'",
                        floor.floor, loot.resource_id
                    ));
                }
            }

            for egg_type_id in &floor.egg_types {
                if self.egg_type(egg_type_id).is_none() {
                    return Err(format!(
                        "Tower floor {} references missing egg type '{}'",
                        floor.floor, egg_type_id
                    ));
                }
            }
        }

        for anomaly in &self.tower_anomalies {
            if anomaly.min_floor == 0
                || anomaly.max_floor < anomaly.min_floor
                || anomaly.visual_index >= 6
            {
                return Err(format!(
                    "Tower anomaly '{}' has an invalid floor range or visual index",
                    anomaly.id
                ));
            }
        }

        for enemy in &self.enemies {
            if enemy.min_floor == 0 || enemy.max_floor < enemy.min_floor {
                return Err(format!("Enemy '{}' has an invalid floor range", enemy.id));
            }
            if enemy.max_hp <= 0 {
                return Err(format!("Enemy '{}' must have positive HP", enemy.id));
            }
            if !(1..=3).contains(&enemy.pack_size) {
                return Err(format!(
                    "Enemy '{}' must use a pack size from 1 to 3",
                    enemy.id
                ));
            }
            for reward in &enemy.rewards {
                if !resource_ids.contains(&reward.resource_id) {
                    return Err(format!(
                        "Enemy '{}' references missing reward resource '{}'",
                        enemy.id, reward.resource_id
                    ));
                }
            }
        }

        for location in &self.tower_special_locations {
            if location.min_floor == 0 || location.max_floor < location.min_floor {
                return Err(format!(
                    "Tower special location '{}' has an invalid floor range",
                    location.id
                ));
            }
            if location.event_ids.is_empty() {
                return Err(format!(
                    "Tower special location '{}' needs at least one event",
                    location.id
                ));
            }
            for event_id in &location.event_ids {
                if self.tower_event(event_id).is_none() {
                    return Err(format!(
                        "Tower special location '{}' references missing event '{}'",
                        location.id, event_id
                    ));
                }
            }
        }
        for event in &self.tower_events {
            validate_resource_stacks(&resource_ids, &event.rewards, "tower event", &event.id)?;
            validate_resource_stacks(
                &resource_ids,
                &event.cargo_costs,
                "tower event cost",
                &event.id,
            )?;
            if !event.enemy_id.is_empty() && self.enemy(&event.enemy_id).is_none() {
                return Err(format!(
                    "Tower event '{}' references missing enemy '{}'",
                    event.id, event.enemy_id
                ));
            }
            if !event.egg_type_id.is_empty() && self.egg_type(&event.egg_type_id).is_none() {
                return Err(format!(
                    "Tower event '{}' references missing egg type '{}'",
                    event.id, event.egg_type_id
                ));
            }
        }
        for hazard in &self.tower_hazards {
            if hazard.min_floor == 0 || hazard.max_floor < hazard.min_floor || hazard.damage < 0 {
                return Err(format!("Tower hazard '{}' has invalid bounds", hazard.id));
            }
            validate_resource_stacks(
                &resource_ids,
                &hazard.counter_rewards,
                "tower hazard",
                &hazard.id,
            )?;
        }
        for contract in &self.tower_contracts {
            if contract.target_amount == 0 || contract.rewards.is_empty() {
                return Err(format!(
                    "Tower contract '{}' needs a target and rewards",
                    contract.id
                ));
            }
            validate_resource_stacks(
                &resource_ids,
                &contract.rewards,
                "tower contract",
                &contract.id,
            )?;
        }
        for contract_id in [
            "balanced",
            "egg_hunt",
            "salvage",
            "scout",
            "push_deeper",
            "safe_run",
        ] {
            if self.tower_contract(contract_id).is_none() {
                return Err(format!("Missing tower contract '{contract_id}'"));
            }
        }

        if self.balance.monster_stat_curves.len() != self.monster_species.len() {
            return Err("Every monster species must have exactly one stat curve".to_owned());
        }
        for species in &self.monster_species {
            let Some(curve) = self.stat_curve(&species.id) else {
                return Err(format!("Missing stat curve for species '{}'", species.id));
            };
            if curve.hp_per_level <= 0
                || curve.attack_per_level < 0
                || curve.defense_per_level < 0
                || curve.speed_per_interval < 0
                || curve.speed_interval == 0
            {
                return Err(format!("Invalid stat curve for species '{}'", species.id));
            }
        }

        for cooldown in &self.balance.combat_cooldowns {
            if cooldown.id != "skill" && cooldown.id != "item" {
                return Err(format!("Unknown combat cooldown '{}'", cooldown.id));
            }
            if cooldown.turns > 10 {
                return Err(format!("Combat cooldown '{}' is too long", cooldown.id));
            }
        }
        for trade in &self.balance.shop_trades {
            if trade.cost.is_empty() || trade.reward.is_empty() {
                return Err(format!("Shop trade '{}' needs cost and reward", trade.id));
            }
            validate_resource_stacks(&resource_ids, &trade.cost, "shop trade", &trade.id)?;
            validate_resource_stacks(&resource_ids, &trade.reward, "shop trade", &trade.id)?;
        }
        for reward in &self.balance.tower_rewards {
            if self.tower_floor(reward.floor).is_none() {
                return Err(format!(
                    "Tower reward references missing floor {}",
                    reward.floor
                ));
            }
            validate_resource_stacks(
                &resource_ids,
                &reward.rewards,
                "tower reward",
                &reward.floor.to_string(),
            )?;
        }

        Ok(())
    }

    fn resource_ids(&self) -> HashSet<String> {
        self.resources
            .iter()
            .map(|resource| resource.id.clone())
            .collect()
    }
}

fn validate_resource_stacks(
    resource_ids: &HashSet<String>,
    stacks: &[crate::data::ResourceAmount],
    kind: &str,
    id: &str,
) -> Result<(), String> {
    for stack in stacks {
        if !resource_ids.contains(&stack.resource_id) || stack.amount <= 0 {
            return Err(format!(
                "{} '{}' has invalid resource amount '{}': {}",
                kind, id, stack.resource_id, stack.amount
            ));
        }
    }
    Ok(())
}

fn build_unique_floor_index<I>(entries: I) -> Result<HashMap<u32, usize>, String>
where
    I: IntoIterator<Item = (u32, usize)>,
{
    let mut index = HashMap::new();
    let mut duplicates = Vec::new();
    for (floor, value) in entries {
        if index.insert(floor, value).is_some() {
            duplicates.push(floor);
        }
    }

    if duplicates.is_empty() {
        Ok(index)
    } else {
        duplicates.sort();
        duplicates.dedup();
        let labels = duplicates
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!("Duplicate tower floor(s): {labels}"))
    }
}

fn build_unique_index<'a, I>(entries: I, kind: &str) -> Result<HashMap<String, usize>, String>
where
    I: IntoIterator<Item = (&'a String, usize)>,
{
    let mut index = HashMap::new();
    let mut duplicates = Vec::new();
    for (id, value) in entries {
        if index.insert(id.clone(), value).is_some() {
            duplicates.push(id.clone());
        }
    }

    if duplicates.is_empty() {
        Ok(index)
    } else {
        duplicates.sort();
        duplicates.dedup();
        Err(format!("Duplicate {kind} id(s): {}", duplicates.join(", ")))
    }
}
