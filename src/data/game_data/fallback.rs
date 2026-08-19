use super::*;

pub(super) fn build() -> GameData {
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
    GameData::from_parts(
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
            guardian_enemy_id: String::new(),
            guardian_egg_type_id: String::new(),
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
            behavior: crate::data::EnemyBehavior::Standard,
            visual: crate::data::DungeonEnemyVisual::Crawler,
            pack_size: 1,
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .expect("fallback Hatchspire data must be valid")
}
