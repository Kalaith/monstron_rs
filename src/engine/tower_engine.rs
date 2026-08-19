mod contracts;
mod discovery;
mod map_gen;
mod map_objects;
mod navigation;
#[cfg(test)]
mod tests;

use crate::data::GameData;
use crate::engine::{monster_engine, town_engine};
use crate::state::{
    DailyCommitment, GameState, ResourceStack, TowerFoundEgg, TowerMapObject, TowerMapObjectKind,
    TowerPendingEvent, TowerRunGoal, TowerRunState,
};
pub use contracts::contract_progress;
use contracts::refresh_contract;
use discovery::{record_enemy_discovery, record_visible_discoveries};
use map_gen::{generate_map, reveal_current_area};
use navigation::explore_direction;

pub struct TowerResult {
    pub summary: String,
    pub encounter: Option<TowerEncounterRequest>,
    pub returned_to_town: bool,
}

pub struct TowerEncounterRequest {
    pub floor: u32,
    pub is_boss: bool,
    pub enemy_id: Option<String>,
}

pub fn start_run(state: &mut GameState, data: &GameData, goal: TowerRunGoal) -> TowerResult {
    if state.tower_run.is_some() {
        return result("The party is already inside the tower.");
    }

    let ready_members = available_party_ids(state);
    if ready_members.is_empty() {
        return result(
            "Assign at least one rested, uncommitted monster to the party before entering the tower. Tap Stable.",
        );
    }

    let start_floor = state
        .tower_progress
        .unlocked_floor
        .max(1)
        .min(max_floor(data));
    let Some(floor) = data.tower_floor(start_floor) else {
        return result(format!(
            "Missing tower floor data for floor {start_floor}. Tap Town to return."
        ));
    };

    for monster_id in ready_members {
        monster_engine::mark_commitment(state, monster_id, DailyCommitment::Tower);
    }

    let seed = tower_seed(state, start_floor, goal, 0);
    let map = generate_map(state, data, start_floor, goal, seed);
    state.tower_run =
        Some(TowerRunState::new(start_floor, floor.pressure_limit, goal).with_map(map));
    record_visible_discoveries(state, data, None);
    let summary = format!(
        "The party enters floor {}: {}. Move through the map to find stairs, eggs, caches, and enemies.",
        floor.floor, floor.name
    );
    state.activity_log.add(state.day, summary.clone());

    result(summary)
}

pub fn ensure_map(state: &mut GameState, data: &GameData) {
    let Some(run) = &state.tower_run else {
        return;
    };
    let needs_map = run.map.is_empty();

    if needs_map {
        let floor = run.current_floor.max(1).min(max_floor(data));
        let goal = run.goal;
        let seed = tower_seed(state, floor, goal, run.rooms_explored);
        let map = generate_map(state, data, floor, goal, seed);
        if let Some(run) = &mut state.tower_run {
            run.current_floor = floor;
            run.map = map;
            run.add_event(format!("Generated a map for floor {floor}."));
        }
        record_visible_discoveries(state, data, None);
        return;
    }

    if let Some(run) = &mut state.tower_run {
        let restored_visibility = run.map.ensure_visibility();
        if restored_visibility || !run.map.is_visible(run.map.player_x, run.map.player_y) {
            reveal_current_area(&mut run.map);
        }
        if restored_visibility {
            run.add_event("Recovered the party's map notes.".to_owned());
        }
    }
    record_visible_discoveries(state, data, None);
}

pub fn move_party(state: &mut GameState, data: &GameData, dx: i32, dy: i32) -> TowerResult {
    if state.tower_run.is_none() {
        return result("No tower run is active. Tap Town to choose a run.");
    }
    if dx == 0 && dy == 0 {
        return result("The party waits and listens.");
    }

    ensure_map(state, data);

    if state
        .tower_run
        .as_ref()
        .is_some_and(|run| run.pressure >= run.pressure_limit)
    {
        return result("The tower is fully awake. Tap CAMP to recover or RETREAT with the cargo.");
    }

    let object = {
        let Some(run) = &mut state.tower_run else {
            return result("No tower run is active. Tap Town to choose a run.");
        };
        if run.map.is_empty() {
            return result(
                "No dungeon map is available. Tap Return to Town and re-enter the tower.",
            );
        }

        let next_x = run.map.player_x as i32 + dx;
        let next_y = run.map.player_y as i32 + dy;
        if next_x < 0 || next_y < 0 || !run.map.is_passable(next_x as u32, next_y as u32) {
            return result("A wall blocks the way. Tap a highlighted adjacent tile.");
        }

        run.map.player_x = next_x as u32;
        run.map.player_y = next_y as u32;
        run.rooms_explored += 1;
        run.camp_cooldown = run.camp_cooldown.saturating_sub(1);
        if run.rooms_explored.is_multiple_of(4) {
            run.pressure = run.pressure.saturating_add(1).min(run.pressure_limit);
        }
        reveal_current_area(&mut run.map);
        run.map
            .object_index_at(run.map.player_x, run.map.player_y)
            .map(|index| run.map.objects.remove(index))
    };

    record_visible_discoveries(state, data, object.as_ref());

    let Some(object) = object else {
        let pressure_warning = state
            .tower_run
            .as_ref()
            .filter(|run| run.pressure + 2 >= run.pressure_limit)
            .map(|run| {
                format!(
                    " Tower pressure is {}/{}.",
                    run.pressure, run.pressure_limit
                )
            })
            .unwrap_or_default();
        return with_contract_refresh(
            result(format!(
                "The party advances through the dungeon.{pressure_warning}"
            )),
            state,
            data,
        );
    };

    let outcome = resolve_map_object(state, data, object);
    with_contract_refresh(outcome, state, data)
}

pub fn explore_party(state: &mut GameState, data: &GameData) -> TowerResult {
    ensure_map(state, data);
    let Some(direction) = state
        .tower_run
        .as_ref()
        .and_then(|run| explore_direction(&run.map, run.goal))
    else {
        return result("No unexplored route is reachable. Tap RETREAT or choose a visible room.");
    };
    move_party(state, data, direction.0, direction.1)
}

pub fn camp_party(state: &mut GameState) -> TowerResult {
    let Some(run) = &state.tower_run else {
        return result("No tower run is active. Tap Town to choose a run.");
    };
    if run.camp_cooldown > 0 {
        return result(format!(
            "The party must travel {} more step(s) before camping again. Tap EXPLORE or RETREAT.",
            run.camp_cooldown
        ));
    }

    let party_ids: Vec<u64> = state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .copied()
        .collect();
    let mut total_healed = 0;
    for monster_id in party_ids {
        if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
            let before = monster.hp;
            monster.hp = (monster.hp + 3).min(monster.max_hp);
            total_healed += monster.hp - before;
        }
    }
    let run = state.tower_run.as_mut().expect("tower run checked above");
    let pressure_reduced = run.pressure.min(2);
    run.pressure -= pressure_reduced;
    run.camp_cooldown = 8;
    let summary = format!(
        "The party makes a brief camp: {total_healed} total HP restored and pressure reduced by {pressure_reduced}."
    );
    run.add_event(summary.clone());
    result(summary)
}

pub fn return_to_town(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(run) = state.tower_run.take() else {
        return result("No tower run is active. Tap Town to choose a run.");
    };

    record_floor_reached(state, data, run.current_floor);

    for stack in &run.cargo {
        state.resources.add(&stack.resource_id, stack.amount);
    }

    let egg_capacity = town_engine::egg_capacity(state);
    let available_egg_slots = egg_capacity.saturating_sub(state.egg_inventory.eggs.len());
    let mut egg_count = 0;
    for found_egg in run.found_eggs.iter().take(available_egg_slots) {
        state.egg_inventory.add_egg(
            found_egg.egg_type_id.clone(),
            found_egg.hatch_days,
            found_egg.origin_floor,
            found_egg.palette_seed,
        );
        egg_count += 1;
    }
    let eggs_left_behind = run.found_eggs.len().saturating_sub(egg_count);

    let cargo_label = cargo_text(data, &run.cargo);
    let mut summary = format!(
        "Returned from floor {} with {} and {} egg(s).",
        run.current_floor, cargo_label, egg_count
    );
    if eggs_left_behind > 0 {
        summary.push_str(&format!(
            " Hatchery capacity left {} egg(s) behind ({}/{}).",
            eggs_left_behind,
            state.egg_inventory.eggs.len(),
            egg_capacity
        ));
    }
    state.activity_log.add(state.day, summary.clone());

    TowerResult {
        summary,
        encounter: None,
        returned_to_town: true,
    }
}

pub fn cargo_text(data: &GameData, cargo: &[ResourceStack]) -> String {
    if cargo.is_empty() {
        return "no materials".to_owned();
    }

    cargo
        .iter()
        .filter(|stack| stack.amount > 0)
        .map(|stack| {
            format!(
                "{} {}",
                stack.amount,
                data.resource_name(&stack.resource_id)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn party_count(state: &GameState) -> usize {
    state
        .monster_roster
        .party_slots
        .iter()
        .filter(|slot| slot.is_some())
        .count()
}

pub fn battle_ready_party_count(state: &GameState) -> usize {
    state
        .monster_roster
        .party_slots
        .iter()
        .filter_map(|slot| state.monster_roster.monster((*slot)?))
        .filter(|monster| monster.is_battle_ready() && state.town.monster_job(monster.id).is_none())
        .count()
}

fn resolve_map_object(
    state: &mut GameState,
    data: &GameData,
    object: TowerMapObject,
) -> TowerResult {
    match object.kind {
        TowerMapObjectKind::Loot => {
            let resource_name = data.resource_name(&object.resource_id).to_owned();
            if let Some(run) = &mut state.tower_run {
                run.add_cargo(&object.resource_id, object.amount);
                let summary = format!(
                    "Found {} {} in a tower cache.",
                    object.amount, resource_name
                );
                run.add_event(summary.clone());
                result(summary)
            } else {
                result("No tower run is active. Tap Town to choose a run.")
            }
        }
        TowerMapObjectKind::Egg => {
            let egg_name = data
                .egg_type(&object.egg_type_id)
                .map(|egg| egg.name.as_str())
                .unwrap_or(object.egg_type_id.as_str())
                .to_owned();
            if let Some(run) = &mut state.tower_run {
                run.found_eggs.push(TowerFoundEgg {
                    egg_type_id: object.egg_type_id,
                    hatch_days: object.hatch_days,
                    origin_floor: run.current_floor,
                    palette_seed: object.palette_seed,
                });
                let summary = format!("Found a {egg_name} in a quiet nest.");
                run.add_event(summary.clone());
                result(summary)
            } else {
                result("No tower run is active. Tap Town to choose a run.")
            }
        }
        TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss => {
            let is_boss = object.kind == TowerMapObjectKind::Boss;
            let floor = state
                .tower_run
                .as_ref()
                .map(|run| run.current_floor)
                .unwrap_or(1);
            let label = if is_boss { "boss" } else { "enemy" };
            let summary = format!("A {label} blocks the tile. Combat starts.");
            if let Some(run) = &mut state.tower_run {
                run.add_event(summary.clone());
            }
            TowerResult {
                summary,
                encounter: Some(TowerEncounterRequest {
                    floor,
                    is_boss,
                    enemy_id: (!object.enemy_id.is_empty()).then_some(object.enemy_id),
                }),
                returned_to_town: false,
            }
        }
        TowerMapObjectKind::Hazard => resolve_hazard(state, data, &object.hazard_id),
        TowerMapObjectKind::SpecialLocation => resolve_special_location(state, data, object),
        TowerMapObjectKind::Stairs => advance_floor(state, data),
        TowerMapObjectKind::Exit => return_to_town(state, data),
    }
}

fn resolve_hazard(state: &mut GameState, data: &GameData, hazard_id: &str) -> TowerResult {
    let Some(hazard) = data.tower_hazard(hazard_id) else {
        return result("The party crosses an unrecorded tower hazard.");
    };
    let countering_monster = state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .filter_map(|id| state.monster_roster.monster(*id))
        .find(|monster| {
            hazard
                .counter_passive
                .is_some_and(|passive| passive == monster.passive)
                || hazard
                    .counter_element
                    .is_some_and(|element| element == monster.element)
        })
        .map(|monster| monster.name.clone());

    let summary = if let Some(monster_name) = countering_monster {
        let mut rewards = Vec::new();
        if let Some(run) = &mut state.tower_run {
            run.stats.hazards_countered += 1;
            for reward in &hazard.counter_rewards {
                run.add_cargo(&reward.resource_id, reward.amount);
                rewards.push(format!(
                    "{} {}",
                    reward.amount,
                    data.resource_name(&reward.resource_id)
                ));
            }
        }
        let reward_text = if rewards.is_empty() {
            String::new()
        } else {
            format!(" Recovered {}.", rewards.join(", "))
        };
        format!(
            "{monster_name} counters {} before it closes on the party.{reward_text}",
            hazard.name
        )
    } else {
        let party_ids: Vec<u64> = state
            .monster_roster
            .party_slots
            .iter()
            .flatten()
            .copied()
            .collect();
        let mut total_damage = 0;
        for monster_id in party_ids {
            if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
                let before = monster.hp;
                monster.hp = (monster.hp - hazard.damage).max(1);
                total_damage += before - monster.hp;
            }
        }
        if let Some(run) = &mut state.tower_run {
            run.pressure = run
                .pressure
                .saturating_add(hazard.pressure)
                .min(run.pressure_limit);
        }
        format!(
            "{} catches the party: {total_damage} total damage and +{} pressure.",
            hazard.name, hazard.pressure
        )
    };
    if let Some(run) = &mut state.tower_run {
        run.add_event(summary.clone());
    }
    result(summary)
}

fn resolve_special_location(
    state: &mut GameState,
    data: &GameData,
    object: TowerMapObject,
) -> TowerResult {
    let Some(location) = data.tower_special_location(&object.special_location_id) else {
        return result("The party finds an unrecorded tower landmark.");
    };
    let mut event_ids = location.event_ids.clone();
    if let Some(index) = event_ids.iter().position(|id| id == &object.event_id) {
        event_ids.rotate_left(index);
    }
    if let Some(run) = &mut state.tower_run {
        run.pending_event = Some(TowerPendingEvent {
            special_location_id: location.id.clone(),
            event_ids,
        });
        let summary = format!(
            "Discovered {}. Choose a visible approach or tap LEAVE.",
            location.name
        );
        run.add_event(summary.clone());
        return result(summary);
    }
    result("No tower run is active. Tap Town to choose a run.")
}

pub fn choose_special_event(state: &mut GameState, data: &GameData, event_id: &str) -> TowerResult {
    let Some(pending) = state
        .tower_run
        .as_ref()
        .and_then(|run| run.pending_event.clone())
    else {
        return result("No landmark decision is waiting. Tap EXPLORE.");
    };
    if !pending
        .event_ids
        .iter()
        .any(|candidate| candidate == event_id)
    {
        return result("That landmark approach is unavailable. Tap a visible choice.");
    }
    if let Some(run) = &mut state.tower_run {
        run.pending_event = None;
        run.stats.landmarks_resolved += 1;
    }
    let outcome = apply_tower_event(state, data, &pending.special_location_id, event_id);
    with_contract_refresh(outcome, state, data)
}

pub fn leave_special_event(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(pending) = state
        .tower_run
        .as_mut()
        .and_then(|run| run.pending_event.take())
    else {
        return result("No landmark decision is waiting. Tap EXPLORE.");
    };
    let location_name = data
        .tower_special_location(&pending.special_location_id)
        .map(|location| location.name.as_str())
        .unwrap_or("the landmark");
    let summary = format!("The party leaves {location_name} undisturbed.");
    if let Some(run) = &mut state.tower_run {
        run.add_event(summary.clone());
    }
    result(summary)
}

fn apply_tower_event(
    state: &mut GameState,
    data: &GameData,
    location_id: &str,
    event_id: &str,
) -> TowerResult {
    let Some(location) = data.tower_special_location(location_id) else {
        return result("The party finds an unrecorded tower landmark.");
    };
    let Some(event) = data.tower_event(event_id) else {
        return result(format!("{} stands silent.", location.name));
    };

    let mut reward_labels = Vec::new();
    let mut found_egg_name = None;
    if let Some(run) = &mut state.tower_run {
        for reward in &event.rewards {
            run.add_cargo(&reward.resource_id, reward.amount);
            reward_labels.push(format!(
                "{} {}",
                reward.amount,
                data.resource_name(&reward.resource_id)
            ));
        }
        if event.pressure_delta < 0 {
            run.pressure = run
                .pressure
                .saturating_sub(event.pressure_delta.unsigned_abs());
        } else {
            run.pressure = run
                .pressure
                .saturating_add(event.pressure_delta as u32)
                .min(run.pressure_limit);
        }
        if event.refresh_camp {
            run.camp_cooldown = 0;
        }
        if event.reveal_map {
            run.map.ensure_visibility();
            for (tile, visibility) in run.map.tiles.iter().zip(run.map.visibility.iter_mut()) {
                if tile.is_passable() && *visibility == crate::state::TowerTileVisibility::Hidden {
                    *visibility = crate::state::TowerTileVisibility::Explored;
                }
            }
        }
        if let Some(egg) = data.egg_type(&event.egg_type_id) {
            run.found_eggs.push(TowerFoundEgg {
                egg_type_id: egg.id.clone(),
                hatch_days: egg.hatch_days,
                origin_floor: run.current_floor,
                palette_seed: event_seed(run.map.seed, &event.id),
            });
            found_egg_name = Some(egg.name.clone());
        }
    }

    let party_ids: Vec<u64> = state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .copied()
        .collect();
    let mut total_healed = 0;
    for monster_id in party_ids {
        if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
            let before = monster.hp;
            monster.hp = (monster.hp + event.party_healing).min(monster.max_hp);
            total_healed += monster.hp - before;
        }
    }

    let mut summary = format!("{} — {}: {}", location.name, event.name, event.narrative);
    if !reward_labels.is_empty() {
        summary.push_str(&format!(" Recovered {}.", reward_labels.join(", ")));
    }
    if total_healed > 0 {
        summary.push_str(&format!(" Restored {total_healed} total HP."));
    }
    if let Some(egg_name) = found_egg_name {
        summary.push_str(&format!(" Recovered a {egg_name}."));
    }
    if event.reveal_map {
        summary.push_str(" The floor's passable routes are now charted.");
    }
    if event.refresh_camp {
        summary.push_str(" The party can CAMP again immediately.");
    }
    if let Some(run) = &mut state.tower_run {
        run.add_event(summary.clone());
    }

    let enemy_id = (!event.enemy_id.is_empty()).then_some(event.enemy_id.clone());
    if let Some(enemy_id) = &enemy_id {
        record_enemy_discovery(state, data, enemy_id);
    }
    TowerResult {
        summary,
        encounter: enemy_id.map(|enemy_id| TowerEncounterRequest {
            floor: state
                .tower_run
                .as_ref()
                .map(|run| run.current_floor)
                .unwrap_or(1),
            is_boss: false,
            enemy_id: Some(enemy_id),
        }),
        returned_to_town: false,
    }
}

fn advance_floor(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(run) = &state.tower_run else {
        return result("No tower run is active. Tap Town to choose a run.");
    };
    let next_floor = (run.current_floor + 1).min(max_floor(data));
    if next_floor == run.current_floor {
        return result("The stairs end at the tower crown.");
    }

    let Some(next_floor_data) = data.tower_floor(next_floor) else {
        return result(format!("Missing tower floor data for floor {next_floor}."));
    };

    record_floor_reached(state, data, next_floor);
    let goal = state
        .tower_run
        .as_ref()
        .map(|run| run.goal)
        .unwrap_or_default();
    let step_count = state
        .tower_run
        .as_ref()
        .map(|run| run.rooms_explored)
        .unwrap_or(0);
    let seed = tower_seed(state, next_floor, goal, step_count);
    let map = generate_map(state, data, next_floor, goal, seed);

    if let Some(run) = &mut state.tower_run {
        run.current_floor = next_floor;
        run.stats.floors_descended += 1;
        run.pressure_limit = next_floor_data.pressure_limit;
        run.map = map;
        let summary = format!(
            "Descended to floor {}: {}. A fresh map unfolds.",
            next_floor_data.floor, next_floor_data.name
        );
        run.add_event(summary.clone());
        return result(summary);
    }

    result("No tower run is active. Tap Town to choose a run.")
}

fn available_party_ids(state: &GameState) -> Vec<u64> {
    state
        .monster_roster
        .party_slots
        .iter()
        .filter_map(|slot| {
            let monster = state.monster_roster.monster((*slot)?)?;
            if monster_engine::can_take_daily_action(state, monster).is_ok() {
                Some(monster.id)
            } else {
                None
            }
        })
        .collect()
}

fn record_floor_reached(state: &mut GameState, data: &GameData, floor: u32) {
    state.tower_progress.best_floor = state.tower_progress.best_floor.max(floor);

    if let Some(floor_data) = data.tower_floor(floor) {
        let unlocked = floor_data.unlocks_floor.max(floor);
        state.tower_progress.unlocked_floor = state
            .tower_progress
            .unlocked_floor
            .max(unlocked)
            .min(max_floor(data));
    }
}

fn tower_seed(state: &GameState, floor: u32, goal: TowerRunGoal, salt: u32) -> u64 {
    0x544F_5745_524D_4150
        ^ u64::from(state.day).wrapping_mul(97)
        ^ u64::from(floor).wrapping_mul(193)
        ^ u64::from(salt).wrapping_mul(389)
        ^ state.egg_inventory.next_id.wrapping_mul(53)
        ^ (goal as u64).wrapping_mul(577)
}

fn event_seed(map_seed: u64, event_id: &str) -> u64 {
    event_id.bytes().fold(map_seed ^ 0xE7E7_5EED, |seed, byte| {
        seed.rotate_left(7) ^ u64::from(byte)
    })
}

fn max_floor(data: &GameData) -> u32 {
    data.tower_floors
        .iter()
        .map(|floor| floor.floor)
        .max()
        .unwrap_or(1)
}

fn result(summary: impl Into<String>) -> TowerResult {
    TowerResult {
        summary: summary.into(),
        encounter: None,
        returned_to_town: false,
    }
}

fn with_contract_refresh(
    mut outcome: TowerResult,
    state: &mut GameState,
    data: &GameData,
) -> TowerResult {
    if let Some(contract_summary) = state
        .tower_run
        .as_mut()
        .and_then(|run| refresh_contract(run, data))
    {
        outcome.summary.push(' ');
        outcome.summary.push_str(&contract_summary);
    }
    outcome
}
