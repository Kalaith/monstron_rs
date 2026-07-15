mod map_gen;
mod map_objects;
#[cfg(test)]
mod tests;

use crate::data::GameData;
use crate::engine::{monster_engine, town_engine};
use crate::state::{
    DailyCommitment, GameState, ResourceStack, TowerFoundEgg, TowerMapObject, TowerMapObjectKind,
    TowerRunGoal, TowerRunState,
};
use map_gen::{generate_map, reveal_current_area};

pub struct TowerResult {
    pub summary: String,
    pub encounter: Option<TowerEncounterRequest>,
    pub returned_to_town: bool,
}

pub struct TowerEncounterRequest {
    pub floor: u32,
    pub is_boss: bool,
}

pub fn start_run(state: &mut GameState, data: &GameData, goal: TowerRunGoal) -> TowerResult {
    if state.tower_run.is_some() {
        return result("The party is already inside the tower.");
    }

    let ready_members = available_party_ids(state);
    if ready_members.is_empty() {
        return result(
            "Assign at least one rested, uncommitted monster to the party before entering the tower.",
        );
    }

    let start_floor = state
        .tower_progress
        .unlocked_floor
        .max(1)
        .min(max_floor(data));
    let Some(floor) = data.tower_floor(start_floor) else {
        return result(format!("Missing tower floor data for floor {start_floor}."));
    };

    for monster_id in ready_members {
        monster_engine::mark_commitment(state, monster_id, DailyCommitment::Tower);
    }

    let seed = tower_seed(state, start_floor, goal, 0);
    let map = generate_map(state, data, start_floor, goal, seed);
    state.tower_run =
        Some(TowerRunState::new(start_floor, floor.pressure_limit, goal).with_map(map));
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
}

pub fn move_party(state: &mut GameState, data: &GameData, dx: i32, dy: i32) -> TowerResult {
    if state.tower_run.is_none() {
        return result("No tower run is active.");
    }
    if dx == 0 && dy == 0 {
        return result("The party waits and listens.");
    }

    ensure_map(state, data);

    let object = {
        let Some(run) = &mut state.tower_run else {
            return result("No tower run is active.");
        };
        if run.map.is_empty() {
            return result("No dungeon map is available.");
        }

        let next_x = run.map.player_x as i32 + dx;
        let next_y = run.map.player_y as i32 + dy;
        if next_x < 0 || next_y < 0 || !run.map.is_passable(next_x as u32, next_y as u32) {
            return result("A wall blocks the way.");
        }

        run.map.player_x = next_x as u32;
        run.map.player_y = next_y as u32;
        run.rooms_explored += 1;
        reveal_current_area(&mut run.map);
        run.map
            .object_index_at(run.map.player_x, run.map.player_y)
            .map(|index| run.map.objects.remove(index))
    };

    let Some(object) = object else {
        return result("The party advances through the dungeon.");
    };

    resolve_map_object(state, data, object)
}

pub fn return_to_town(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(run) = state.tower_run.take() else {
        return result("No tower run is active.");
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
                result("No tower run is active.")
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
                result("No tower run is active.")
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
                encounter: Some(TowerEncounterRequest { floor, is_boss }),
                returned_to_town: false,
            }
        }
        TowerMapObjectKind::Stairs => advance_floor(state, data),
        TowerMapObjectKind::Exit => return_to_town(state, data),
    }
}

fn advance_floor(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(run) = &state.tower_run else {
        return result("No tower run is active.");
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
        run.pressure_limit = next_floor_data.pressure_limit;
        run.map = map;
        let summary = format!(
            "Descended to floor {}: {}. A fresh map unfolds.",
            next_floor_data.floor, next_floor_data.name
        );
        run.add_event(summary.clone());
        return result(summary);
    }

    result("No tower run is active.")
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
