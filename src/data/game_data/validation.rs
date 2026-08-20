//! Cross-reference and balance validation for loaded game content.

use std::collections::HashSet;

use super::GameData;

pub(super) fn validate(data: &GameData) -> Result<(), String> {
    if data.species(&data.config.starter_species_id).is_none() {
        return Err(format!(
            "Starter species '{}' does not exist",
            data.config.starter_species_id
        ));
    }

    let resource_ids = resource_ids(data);
    for building in &data.buildings {
        for cost in &building.upgrade_cost {
            if !resource_ids.contains(&cost.resource_id) {
                return Err(format!(
                    "Building '{}' references missing resource '{}'",
                    building.id, cost.resource_id
                ));
            }
        }
    }

    for egg in &data.egg_types {
        for species_id in &egg.possible_species {
            if data.species(species_id).is_none() {
                return Err(format!(
                    "Egg '{}' references missing species '{}'",
                    egg.id, species_id
                ));
            }
        }
    }

    for floor in &data.tower_floors {
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
        if !floor.guardian_enemy_id.is_empty() {
            let Some(guardian) = data.enemy(&floor.guardian_enemy_id) else {
                return Err(format!(
                    "Tower floor {} references missing guardian '{}'",
                    floor.floor, floor.guardian_enemy_id
                ));
            };
            if !guardian.is_boss
                || guardian.min_floor > floor.floor
                || guardian.max_floor < floor.floor
            {
                return Err(format!(
                    "Tower floor {} guardian '{}' is not an eligible boss",
                    floor.floor, floor.guardian_enemy_id
                ));
            }
            if floor.guardian_egg_type_id.is_empty()
                || data.egg_type(&floor.guardian_egg_type_id).is_none()
            {
                return Err(format!(
                    "Tower floor {} guardian needs a valid egg reward '{}'",
                    floor.floor, floor.guardian_egg_type_id
                ));
            }
        }

        for egg_type_id in &floor.egg_types {
            if data.egg_type(egg_type_id).is_none() {
                return Err(format!(
                    "Tower floor {} references missing egg type '{}'",
                    floor.floor, egg_type_id
                ));
            }
        }
    }

    for anomaly in &data.tower_anomalies {
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

    for enemy in &data.enemies {
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

    for location in &data.tower_special_locations {
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
            if data.tower_event(event_id).is_none() {
                return Err(format!(
                    "Tower special location '{}' references missing event '{}'",
                    location.id, event_id
                ));
            }
        }
    }

    for event in &data.tower_events {
        validate_resource_stacks(&resource_ids, &event.rewards, "tower event", &event.id)?;
        validate_resource_stacks(
            &resource_ids,
            &event.cargo_costs,
            "tower event cost",
            &event.id,
        )?;
        if !event.enemy_id.is_empty() && data.enemy(&event.enemy_id).is_none() {
            return Err(format!(
                "Tower event '{}' references missing enemy '{}'",
                event.id, event.enemy_id
            ));
        }
        if !event.egg_type_id.is_empty() && data.egg_type(&event.egg_type_id).is_none() {
            return Err(format!(
                "Tower event '{}' references missing egg type '{}'",
                event.id, event.egg_type_id
            ));
        }
    }

    for hazard in &data.tower_hazards {
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

    for contract in &data.tower_contracts {
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
        if data.tower_contract(contract_id).is_none() {
            return Err(format!("Missing tower contract '{contract_id}'"));
        }
    }

    if data.balance.monster_stat_curves.len() != data.monster_species.len() {
        return Err("Every monster species must have exactly one stat curve".to_owned());
    }
    for species in &data.monster_species {
        let Some(curve) = data.stat_curve(&species.id) else {
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

    for cooldown in &data.balance.combat_cooldowns {
        if cooldown.id != "skill" && cooldown.id != "item" {
            return Err(format!("Unknown combat cooldown '{}'", cooldown.id));
        }
        if cooldown.turns > 10 {
            return Err(format!("Combat cooldown '{}' is too long", cooldown.id));
        }
    }
    for trade in &data.balance.shop_trades {
        if trade.cost.is_empty() || trade.reward.is_empty() {
            return Err(format!("Shop trade '{}' needs cost and reward", trade.id));
        }
        validate_resource_stacks(&resource_ids, &trade.cost, "shop trade", &trade.id)?;
        validate_resource_stacks(&resource_ids, &trade.reward, "shop trade", &trade.id)?;
    }
    for reward in &data.balance.tower_rewards {
        if data.tower_floor(reward.floor).is_none() {
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

fn resource_ids(data: &GameData) -> HashSet<String> {
    data.resources
        .iter()
        .map(|resource| resource.id.clone())
        .collect()
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
