use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::data::{
    BalanceData, BuildingDefinition, EggTypeDefinition, Element, EnemyDefinition, GameConfig,
    MonsterRole, MonsterSpeciesDefinition, NpcDefinition, PassiveSkill, ResourceDefinition,
    ShopTradeDefinition, Temperament, TowerFloorDefinition, TowerRewardDefinition, TownSkill,
};

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
            npcs,
            resource_index: HashMap::new(),
            building_index: HashMap::new(),
            species_index: HashMap::new(),
            egg_index: HashMap::new(),
            tower_floor_index: HashMap::new(),
            enemy_index: HashMap::new(),
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
        let balance = BalanceData {
            monster_stat_curves: vec![crate::data::MonsterStatCurveDefinition {
                species_id: "slime".to_owned(),
                hp_per_level: 3,
                attack_per_level: 1,
                defense_per_level: 1,
                speed_per_interval: 1,
                speed_interval: 2,
            }],
            combat_cooldowns: vec![
                crate::data::CombatCooldownDefinition {
                    id: "skill".to_owned(),
                    turns: 0,
                },
                crate::data::CombatCooldownDefinition {
                    id: "item".to_owned(),
                    turns: 0,
                },
            ],
            shop_trades: vec![
                crate::data::ShopTradeDefinition {
                    id: "buy_herbs".to_owned(),
                    label: "Buy Herbs".to_owned(),
                    detail: "Restock egg warming and grove supplies.".to_owned(),
                    cost: vec![crate::data::ResourceAmount {
                        resource_id: "coins".to_owned(),
                        amount: 6,
                    }],
                    reward: vec![crate::data::ResourceAmount {
                        resource_id: "herbs".to_owned(),
                        amount: 3,
                    }],
                },
                crate::data::ShopTradeDefinition {
                    id: "buy_stone".to_owned(),
                    label: "Buy Stone".to_owned(),
                    detail: "Convert coins into upgrade materials.".to_owned(),
                    cost: vec![crate::data::ResourceAmount {
                        resource_id: "coins".to_owned(),
                        amount: 8,
                    }],
                    reward: vec![crate::data::ResourceAmount {
                        resource_id: "stone".to_owned(),
                        amount: 4,
                    }],
                },
                crate::data::ShopTradeDefinition {
                    id: "sell_herbs".to_owned(),
                    label: "Sell Herbs".to_owned(),
                    detail: "Turn spare herbs back into coins.".to_owned(),
                    cost: vec![crate::data::ResourceAmount {
                        resource_id: "herbs".to_owned(),
                        amount: 2,
                    }],
                    reward: vec![crate::data::ResourceAmount {
                        resource_id: "coins".to_owned(),
                        amount: 5,
                    }],
                },
            ],
            tower_rewards: vec![crate::data::TowerRewardDefinition {
                floor: 1,
                rewards: vec![crate::data::ResourceAmount {
                    resource_id: "coins".to_owned(),
                    amount: 2,
                }],
            }],
        };
        Self::from_parts(
            GameConfig {
                save_version: 1,
                starting_day: 1,
                starter_species_id: "slime".to_owned(),
                starter_name: "Pip".to_owned(),
                starting_log: vec![
                    "A ruined tower rises above the camp.".to_owned(),
                    "Pip the slime waits beside a cold hatchery brazier.".to_owned(),
                ],
            },
            balance,
            vec![
                ResourceDefinition {
                    id: "coins".to_owned(),
                    name: "Coins".to_owned(),
                    starting_amount: 30,
                },
                ResourceDefinition {
                    id: "wood".to_owned(),
                    name: "Wood".to_owned(),
                    starting_amount: 12,
                },
                ResourceDefinition {
                    id: "stone".to_owned(),
                    name: "Stone".to_owned(),
                    starting_amount: 8,
                },
                ResourceDefinition {
                    id: "herbs".to_owned(),
                    name: "Herbs".to_owned(),
                    starting_amount: 5,
                },
            ],
            vec![BuildingDefinition {
                id: "camp".to_owned(),
                name: "Tower Camp".to_owned(),
                description: "A small shelter where each new day begins.".to_owned(),
                starting_level: 1,
                max_level: 3,
                upgrade_cost: Vec::new(),
            }],
            vec![MonsterSpeciesDefinition {
                id: "slime".to_owned(),
                name: "Slime".to_owned(),
                element: Element::Water,
                temperament: Temperament::Loyal,
                role: MonsterRole::Scout,
                passive: PassiveSkill::FindsSmallLoot,
                town_skill: TownSkill::HatcheryHelper,
                base_hp: 18,
                base_attack: 4,
                base_defense: 3,
                base_speed: 6,
            }],
            Vec::new(),
            vec![TowerFloorDefinition {
                floor: 1,
                name: "Tower Edge".to_owned(),
                theme: "Broken stairways and mossy rooms.".to_owned(),
                enemy_hint: "Wary tower vermin".to_owned(),
                loot: vec![
                    crate::data::ResourceAmount {
                        resource_id: "wood".to_owned(),
                        amount: 4,
                    },
                    crate::data::ResourceAmount {
                        resource_id: "herbs".to_owned(),
                        amount: 2,
                    },
                ],
                egg_types: Vec::new(),
                pressure_limit: 8,
                unlocks_floor: 2,
                is_boss_floor: false,
            }],
            vec![EnemyDefinition {
                id: "moss_mite".to_owned(),
                name: "Moss Mite".to_owned(),
                description: "A skittering tower pest.".to_owned(),
                min_floor: 1,
                max_floor: 3,
                is_boss: false,
                max_hp: 14,
                attack: 4,
                defense: 1,
                speed: 5,
                xp_reward: 5,
                rewards: vec![crate::data::ResourceAmount {
                    resource_id: "coins".to_owned(),
                    amount: 3,
                }],
            }],
            Vec::new(),
        )
        .expect("fallback Hatchspire data must be valid")
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

        for enemy in &self.enemies {
            if enemy.min_floor == 0 || enemy.max_floor < enemy.min_floor {
                return Err(format!("Enemy '{}' has an invalid floor range", enemy.id));
            }
            if enemy.max_hp <= 0 {
                return Err(format!("Enemy '{}' must have positive HP", enemy.id));
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
