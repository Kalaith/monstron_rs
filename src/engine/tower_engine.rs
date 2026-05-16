use crate::data::{GameData, ResourceAmount, TowerFloorDefinition};
use crate::state::{GameState, ResourceStack, TowerFoundEgg, TowerRunState};

pub struct TowerResult {
    pub summary: String,
}

pub fn start_run(state: &mut GameState, data: &GameData) -> TowerResult {
    if state.tower_run.is_some() {
        return TowerResult {
            summary: "The party is already inside the tower.".to_owned(),
        };
    }

    let party_count = party_count(state);
    if party_count == 0 {
        return TowerResult {
            summary: "Assign at least one monster to the party before entering the tower."
                .to_owned(),
        };
    }

    let start_floor = state
        .tower_progress
        .unlocked_floor
        .max(1)
        .min(max_floor(data));
    let Some(floor) = data.tower_floor(start_floor) else {
        return TowerResult {
            summary: format!("Missing tower floor data for floor {start_floor}."),
        };
    };

    state.tower_run = Some(TowerRunState::new(start_floor, floor.pressure_limit));
    let summary = format!(
        "The party enters floor {}: {}. Pressure limit {}.",
        floor.floor, floor.name, floor.pressure_limit
    );
    state.activity_log.add(state.day, summary.clone());

    TowerResult { summary }
}

pub fn explore_room(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(active_run) = &state.tower_run else {
        return TowerResult {
            summary: "No tower run is active.".to_owned(),
        };
    };

    if active_run.pressure >= active_run.pressure_limit {
        return TowerResult {
            summary: "Tower pressure is maxed. Return to town with the current loot.".to_owned(),
        };
    }

    let current_floor = active_run.current_floor;
    let Some(floor) = data.tower_floor(current_floor) else {
        return TowerResult {
            summary: format!("Missing tower floor data for floor {current_floor}."),
        };
    };

    let room_number = active_run.rooms_explored + 1;
    let event_seed = tower_seed(state, floor, room_number);
    let event_kind = choose_event(floor, room_number, event_seed, max_floor(data));
    let mut reached_floor = None;
    let mut summary = String::new();

    if let Some(run) = &mut state.tower_run {
        run.rooms_explored += 1;
        run.pressure = (run.pressure + 1).min(run.pressure_limit);

        match event_kind {
            TowerEvent::Loot => {
                if let Some(gain) = loot_gain(floor, event_seed) {
                    run.add_cargo(&gain.resource_id, gain.amount);
                    summary = format!(
                        "Floor {} cache: gained {} {}.",
                        floor.floor,
                        gain.amount,
                        data.resource_name(&gain.resource_id)
                    );
                } else {
                    summary = format!("Floor {} has no loot cache.", floor.floor);
                }
                run.add_event(summary.clone());
            }
            TowerEvent::Egg => {
                if let Some(found_egg) = found_egg(floor, data, event_seed) {
                    let egg_name = data
                        .egg_type(&found_egg.egg_type_id)
                        .map(|egg_type| egg_type.name.as_str())
                        .unwrap_or(found_egg.egg_type_id.as_str())
                        .to_owned();
                    run.found_eggs.push(found_egg);
                    summary = format!("Found a {egg_name} in a sheltered nest.");
                } else if let Some(gain) = loot_gain(floor, event_seed.rotate_left(5)) {
                    run.add_cargo(&gain.resource_id, gain.amount);
                    summary = format!(
                        "The nest is empty, but the party recovers {} {}.",
                        gain.amount,
                        data.resource_name(&gain.resource_id)
                    );
                } else {
                    summary = "The party finds an empty nest.".to_owned();
                }
                run.add_event(summary.clone());
            }
            TowerEvent::Enemy => {
                let extra_pressure = (2 + floor.floor / 5).min(4);
                run.pressure = (run.pressure + extra_pressure).min(run.pressure_limit);
                summary = format!(
                    "{} blocks the route. The party avoids combat; pressure rises by {}.",
                    floor.enemy_hint, extra_pressure
                );
                run.add_event(summary.clone());
            }
            TowerEvent::Exit => {
                let next_floor = (floor.floor + 1).min(max_floor(data));
                if let Some(next_floor_data) = data.tower_floor(next_floor) {
                    run.current_floor = next_floor;
                    run.pressure_limit = next_floor_data.pressure_limit;
                    summary = format!(
                        "Found a deeper stair and reached floor {}: {}.",
                        next_floor_data.floor, next_floor_data.name
                    );
                    reached_floor = Some(next_floor);
                } else {
                    summary = "The stair collapses before the party can descend.".to_owned();
                }
                run.add_event(summary.clone());
            }
            TowerEvent::Boss => {
                let extra_pressure = 3;
                run.pressure = (run.pressure + extra_pressure).min(run.pressure_limit);
                if let Some(found_egg) = found_egg(floor, data, event_seed ^ 0xB055) {
                    let egg_name = data
                        .egg_type(&found_egg.egg_type_id)
                        .map(|egg_type| egg_type.name.as_str())
                        .unwrap_or(found_egg.egg_type_id.as_str())
                        .to_owned();
                    run.found_eggs.push(found_egg);
                    summary = format!(
                        "The {} stirs. The party claims a {} and retreats from combat.",
                        floor.enemy_hint, egg_name
                    );
                } else {
                    summary = format!(
                        "The {} stirs. Combat will resolve this guardian in Phase 5.",
                        floor.enemy_hint
                    );
                }
                reached_floor = Some(floor.floor);
                run.add_event(summary.clone());
            }
        }
    }

    if let Some(floor) = reached_floor {
        record_floor_reached(state, data, floor);
    }

    TowerResult { summary }
}

pub fn return_to_town(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(run) = state.tower_run.take() else {
        return TowerResult {
            summary: "No tower run is active.".to_owned(),
        };
    };

    record_floor_reached(state, data, run.current_floor);

    for stack in &run.cargo {
        state.resources.add(&stack.resource_id, stack.amount);
    }

    let mut egg_count = 0;
    for found_egg in &run.found_eggs {
        state.egg_inventory.add_egg(
            found_egg.egg_type_id.clone(),
            found_egg.hatch_days,
            found_egg.origin_floor,
            found_egg.palette_seed,
        );
        egg_count += 1;
    }

    let cargo_label = cargo_text(data, &run.cargo);
    let summary = format!(
        "Returned from floor {} with {} and {} egg(s).",
        run.current_floor, cargo_label, egg_count
    );
    state.activity_log.add(state.day, summary.clone());

    TowerResult { summary }
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

fn choose_event(
    floor: &TowerFloorDefinition,
    room_number: u32,
    event_seed: u64,
    highest_floor: u32,
) -> TowerEvent {
    if floor.is_boss_floor && room_number >= 3 {
        return TowerEvent::Boss;
    }

    if floor.floor < highest_floor && room_number % 3 == 0 {
        return TowerEvent::Exit;
    }

    match event_seed % 5 {
        0 | 3 => TowerEvent::Loot,
        1 => TowerEvent::Egg,
        2 => TowerEvent::Enemy,
        _ if floor.floor < highest_floor && room_number > 1 => TowerEvent::Exit,
        _ => TowerEvent::Loot,
    }
}

fn loot_gain(floor: &TowerFloorDefinition, seed: u64) -> Option<ResourceAmount> {
    let base = floor.loot.get((seed as usize) % floor.loot.len().max(1))?;
    let bonus = ((seed >> 8) % 3) as i32;
    Some(ResourceAmount {
        resource_id: base.resource_id.clone(),
        amount: base.amount + bonus,
    })
}

fn found_egg(floor: &TowerFloorDefinition, data: &GameData, seed: u64) -> Option<TowerFoundEgg> {
    let egg_type_id = floor
        .egg_types
        .get((seed as usize) % floor.egg_types.len().max(1))?;
    let egg_type = data.egg_type(egg_type_id)?;

    Some(TowerFoundEgg {
        egg_type_id: egg_type.id.clone(),
        hatch_days: egg_type.hatch_days,
        origin_floor: floor.floor,
        palette_seed: 0x7000_0000 ^ seed ^ (u64::from(floor.floor) << 24),
    })
}

fn tower_seed(state: &GameState, floor: &TowerFloorDefinition, room_number: u32) -> u64 {
    0x5470_5745
        ^ u64::from(state.day) * 97
        ^ u64::from(floor.floor) * 193
        ^ u64::from(room_number) * 389
        ^ state.egg_inventory.next_id * 53
}

fn max_floor(data: &GameData) -> u32 {
    data.tower_floors
        .iter()
        .map(|floor| floor.floor)
        .max()
        .unwrap_or(1)
}

enum TowerEvent {
    Loot,
    Egg,
    Enemy,
    Exit,
    Boss,
}
