//! Lookup-index construction for validated game content.

use std::collections::HashMap;

use super::GameData;

pub(super) fn build(data: &mut GameData) -> Result<(), String> {
    data.resource_index = build_unique_index(
        data.resources
            .iter()
            .enumerate()
            .map(|(index, resource)| (&resource.id, index)),
        "resource",
    )?;
    data.building_index = build_unique_index(
        data.buildings
            .iter()
            .enumerate()
            .map(|(index, building)| (&building.id, index)),
        "building",
    )?;
    data.species_index = build_unique_index(
        data.monster_species
            .iter()
            .enumerate()
            .map(|(index, species)| (&species.id, index)),
        "monster species",
    )?;
    data.egg_index = build_unique_index(
        data.egg_types
            .iter()
            .enumerate()
            .map(|(index, egg)| (&egg.id, index)),
        "egg type",
    )?;
    data.tower_floor_index = build_unique_floor_index(
        data.tower_floors
            .iter()
            .enumerate()
            .map(|(index, floor)| (floor.floor, index)),
    )?;
    data.enemy_index = build_unique_index(
        data.enemies
            .iter()
            .enumerate()
            .map(|(index, enemy)| (&enemy.id, index)),
        "enemy",
    )?;
    data.tower_special_location_index = build_unique_index(
        data.tower_special_locations
            .iter()
            .enumerate()
            .map(|(index, location)| (&location.id, index)),
        "tower special location",
    )?;
    data.tower_event_index = build_unique_index(
        data.tower_events
            .iter()
            .enumerate()
            .map(|(index, event)| (&event.id, index)),
        "tower event",
    )?;
    data.tower_hazard_index = build_unique_index(
        data.tower_hazards
            .iter()
            .enumerate()
            .map(|(index, hazard)| (&hazard.id, index)),
        "tower hazard",
    )?;
    data.tower_contract_index = build_unique_index(
        data.tower_contracts
            .iter()
            .enumerate()
            .map(|(index, contract)| (&contract.id, index)),
        "tower contract",
    )?;
    data.tower_anomaly_index = build_unique_index(
        data.tower_anomalies
            .iter()
            .enumerate()
            .map(|(index, anomaly)| (&anomaly.id, index)),
        "tower anomaly",
    )?;
    data.npc_index = build_unique_index(
        data.npcs
            .iter()
            .enumerate()
            .map(|(index, npc)| (&npc.id, index)),
        "npc",
    )?;
    data.stat_curve_index = build_unique_index(
        data.balance
            .monster_stat_curves
            .iter()
            .enumerate()
            .map(|(index, curve)| (&curve.species_id, index)),
        "monster stat curve",
    )?;
    data.cooldown_index = build_unique_index(
        data.balance
            .combat_cooldowns
            .iter()
            .enumerate()
            .map(|(index, cooldown)| (&cooldown.id, index)),
        "combat cooldown",
    )?;
    data.shop_trade_index = build_unique_index(
        data.balance
            .shop_trades
            .iter()
            .enumerate()
            .map(|(index, trade)| (&trade.id, index)),
        "shop trade",
    )?;
    data.tower_reward_index = build_unique_floor_index(
        data.balance
            .tower_rewards
            .iter()
            .enumerate()
            .map(|(index, reward)| (reward.floor, index)),
    )?;
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
