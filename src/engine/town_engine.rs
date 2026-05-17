use crate::data::{BuildingDefinition, GameData};
use crate::state::GameState;

const SHOP_ID: &str = "shop";
const STABLE_ID: &str = "stable";
const HATCHERY_ID: &str = "hatchery";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShopTrade {
    BuyHerbs,
    BuyStone,
    SellHerbs,
}

pub struct TownResult {
    pub summary: String,
}

pub fn advance_building(state: &mut GameState, data: &GameData, building_id: &str) -> TownResult {
    let Some(building) = data.building(building_id) else {
        return TownResult {
            summary: format!("Unknown building plan: {building_id}."),
        };
    };

    let current_level = state.town.building_level(building_id);
    if current_level >= building.max_level {
        return TownResult {
            summary: format!("{} is already at max level.", building.name),
        };
    }

    let next_level = current_level + 1;
    let cost = scaled_cost(building, next_level);
    if let Err(error) = state.resources.spend(&cost) {
        return TownResult {
            summary: format!("{} needs {}.", building.name, cost_label(data, &error)),
        };
    }

    state.town.set_building_level(building_id, next_level);
    let summary = if current_level == 0 {
        format!("Built {} at level {}.", building.name, next_level)
    } else {
        format!("Upgraded {} to level {}.", building.name, next_level)
    };
    state.activity_log.add(state.day, summary.clone());

    TownResult { summary }
}

pub fn scavenge_supplies(state: &mut GameState) -> TownResult {
    let gains = [("coins", 8), ("wood", 6), ("stone", 4), ("herbs", 2)];
    for (resource_id, amount) in gains {
        state.resources.add(resource_id, amount);
    }

    let summary = "The camp scavenges 8 coins, 6 wood, 4 stone, and 2 herbs.".to_owned();
    state.activity_log.add(state.day, summary.clone());
    TownResult { summary }
}

pub fn trade_shop(state: &mut GameState, data: &GameData, trade: ShopTrade) -> TownResult {
    if state.town.building_level(SHOP_ID) == 0 {
        return TownResult {
            summary: "Build the shop before trading supplies.".to_owned(),
        };
    }

    let (cost, gain, success) = match trade {
        ShopTrade::BuyHerbs => (
            vec![("coins".to_owned(), 6)],
            vec![("herbs".to_owned(), 3)],
            "Bought 3 herbs for 6 coins.",
        ),
        ShopTrade::BuyStone => (
            vec![("coins".to_owned(), 8)],
            vec![("stone".to_owned(), 4)],
            "Bought 4 stone for 8 coins.",
        ),
        ShopTrade::SellHerbs => (
            vec![("herbs".to_owned(), 2)],
            vec![("coins".to_owned(), 5)],
            "Sold 2 herbs for 5 coins.",
        ),
    };

    if let Err(missing) = state.resources.spend(&cost) {
        return TownResult {
            summary: format!("Trade needs {}.", cost_label(data, &missing)),
        };
    }

    for (resource_id, amount) in gain {
        state.resources.add(&resource_id, amount);
    }

    let summary = success.to_owned();
    state.activity_log.add(state.day, summary.clone());
    TownResult { summary }
}

pub fn greet_npc(state: &mut GameState, data: &GameData, npc_id: &str) -> TownResult {
    let Some(npc) = data.npc(npc_id) else {
        return TownResult {
            summary: format!("Unknown NPC: {npc_id}."),
        };
    };

    let friendship = {
        let friendship = state.npc_friendship_mut(npc_id);
        *friendship += 1;
        *friendship
    };
    let summary = format!(
        "You check in with {}. Friendship is now {}.",
        npc.name, friendship
    );
    state.activity_log.add(state.day, summary.clone());

    TownResult { summary }
}

pub fn town_rank(state: &GameState) -> u32 {
    let building_score: u32 = state
        .town
        .buildings
        .iter()
        .map(|building| building.level)
        .sum();
    let monster_score = state.monster_roster.monsters.len() as u32;
    let tower_score = state.tower_progress.best_floor / 2;

    1 + (building_score + monster_score + tower_score) / 4
}

pub fn monster_capacity(state: &GameState) -> usize {
    match state.town.building_level(STABLE_ID) {
        0 => 3,
        1 => 6,
        2 => 9,
        _ => 12,
    }
}

pub fn egg_capacity(state: &GameState) -> usize {
    match state.town.building_level(HATCHERY_ID) {
        0 => 0,
        1 => 3,
        2 => 5,
        _ => 8,
    }
}

pub fn has_monster_capacity(state: &GameState) -> bool {
    state.monster_roster.monsters.len() < monster_capacity(state)
}

pub fn has_egg_capacity(state: &GameState) -> bool {
    state.egg_inventory.eggs.len() < egg_capacity(state)
}

pub fn next_cost(data: &GameData, building_id: &str, current_level: u32) -> Vec<(String, i32)> {
    let Some(building) = data.building(building_id) else {
        return Vec::new();
    };

    if current_level >= building.max_level {
        Vec::new()
    } else {
        scaled_cost(building, current_level + 1)
    }
}

pub fn cost_text(data: &GameData, cost: &[(String, i32)]) -> String {
    if cost.is_empty() {
        return "No cost".to_owned();
    }

    cost.iter()
        .map(|(resource_id, amount)| format!("{} {}", amount, data.resource_name(resource_id)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn scaled_cost(building: &BuildingDefinition, next_level: u32) -> Vec<(String, i32)> {
    building
        .upgrade_cost
        .iter()
        .map(|cost| {
            (
                cost.resource_id.clone(),
                cost.amount * i32::try_from(next_level).unwrap_or(i32::MAX),
            )
        })
        .collect()
}

fn cost_label(data: &GameData, missing: &[(String, i32)]) -> String {
    cost_text(data, missing)
}
