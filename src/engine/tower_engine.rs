use crate::data::{GameData, MonsterRole, PassiveSkill, ResourceAmount, TowerFloorDefinition};
use crate::engine::{monster_engine, town_engine};
use crate::state::{
    DailyCommitment, GameState, ResourceStack, TowerFoundEgg, TowerRunGoal, TowerRunState,
};

pub struct TowerResult {
    pub summary: String,
    pub encounter: Option<TowerEncounterRequest>,
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

    state.tower_run = Some(TowerRunState::new(start_floor, floor.pressure_limit, goal));
    let summary = format!(
        "The party enters floor {}: {} for a {} run. Pressure limit {}.",
        floor.floor, floor.name, goal, floor.pressure_limit
    );
    state.activity_log.add(state.day, summary.clone());

    result(summary)
}

pub fn explore_room(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(active_run) = &state.tower_run else {
        return result("No tower run is active.");
    };

    if active_run.pressure >= active_run.pressure_limit {
        return result("Tower pressure is maxed. Return to town with the current loot.");
    }

    let current_floor = active_run.current_floor;
    let Some(floor) = data.tower_floor(current_floor) else {
        return result(format!(
            "Missing tower floor data for floor {current_floor}."
        ));
    };

    let room_number = active_run.rooms_explored + 1;
    let goal = active_run.goal;
    let base_pressure = pressure_gain(goal, room_number, state, data);
    let projected_pressure = (active_run.pressure + base_pressure).min(active_run.pressure_limit);
    let pressure_tier = pressure_tier(projected_pressure, active_run.pressure_limit);
    let party_profile = party_profile(state, data);
    let event_seed = tower_seed(state, floor, room_number);
    let event_kind = choose_event(
        floor,
        room_number,
        event_seed,
        max_floor(data),
        goal,
        pressure_tier,
        &party_profile,
    );
    let mut reached_floor = None;
    let mut encounter = None;
    let mut summary = String::new();

    if let Some(run) = &mut state.tower_run {
        run.rooms_explored += 1;
        run.pressure = (run.pressure + base_pressure).min(run.pressure_limit);

        match event_kind {
            TowerEvent::Loot => {
                if let Some(gain) =
                    loot_gain(floor, event_seed, goal, pressure_tier, &party_profile)
                {
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
                if let Some(found_egg) =
                    found_egg(floor, data, event_seed, goal, pressure_tier, &party_profile)
                {
                    let egg_name = data
                        .egg_type(&found_egg.egg_type_id)
                        .map(|egg_type| egg_type.name.as_str())
                        .unwrap_or(found_egg.egg_type_id.as_str())
                        .to_owned();
                    run.found_eggs.push(found_egg);
                    summary = format!("Found a {egg_name} in a sheltered nest.");
                } else if let Some(gain) = loot_gain(
                    floor,
                    event_seed.rotate_left(5),
                    goal,
                    pressure_tier,
                    &party_profile,
                ) {
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
                let extra_pressure = enemy_pressure_gain(goal, floor.floor, &party_profile);
                run.pressure = (run.pressure + extra_pressure).min(run.pressure_limit);
                summary = format!(
                    "{} blocks the route. Combat starts; pressure rises by {}.",
                    floor.enemy_hint, extra_pressure
                );
                encounter = Some(TowerEncounterRequest {
                    floor: floor.floor,
                    is_boss: false,
                });
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
                summary = format!("The {} stirs. Boss combat starts.", floor.enemy_hint);
                encounter = Some(TowerEncounterRequest {
                    floor: floor.floor,
                    is_boss: true,
                });
                reached_floor = Some(floor.floor);
                run.add_event(summary.clone());
            }
        }
        if let Some(flavor) = pressure_flavor(run.pressure, run.pressure_limit) {
            run.add_event(flavor.to_owned());
        }
    }

    if let Some(floor) = reached_floor {
        record_floor_reached(state, data, floor);
    }

    TowerResult { summary, encounter }
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

    result(summary)
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

fn choose_event(
    floor: &TowerFloorDefinition,
    room_number: u32,
    event_seed: u64,
    highest_floor: u32,
    goal: TowerRunGoal,
    pressure_tier: u32,
    profile: &TowerPartyProfile,
) -> TowerEvent {
    if floor.is_boss_floor && room_number >= 3 {
        return TowerEvent::Boss;
    }

    if pressure_tier >= 3 {
        return TowerEvent::Enemy;
    }

    if goal == TowerRunGoal::PushDeeper && floor.floor < highest_floor && room_number % 2 == 0 {
        return TowerEvent::Exit;
    }

    if floor.floor < highest_floor && room_number % 3 == 0 {
        return TowerEvent::Exit;
    }

    let roll = (event_seed + u64::from(profile.egg_sense) * 5 + u64::from(profile.salvage)) % 12;
    match goal {
        TowerRunGoal::EggHunt if matches!(roll, 0..=4) => TowerEvent::Egg,
        TowerRunGoal::Salvage if matches!(roll, 0..=6) => TowerEvent::Loot,
        TowerRunGoal::Scout if matches!(roll, 0..=2) => TowerEvent::Loot,
        TowerRunGoal::Scout if floor.floor < highest_floor && matches!(roll, 3..=5) => {
            TowerEvent::Exit
        }
        TowerRunGoal::SafeRun if matches!(roll, 0..=5) => TowerEvent::Loot,
        _ if pressure_tier >= 2 && matches!(roll, 0..=2) => TowerEvent::Egg,
        _ if pressure_tier >= 1 && matches!(roll, 3..=5) => TowerEvent::Enemy,
        _ if matches!(roll, 0 | 3 | 7) => TowerEvent::Loot,
        _ if matches!(roll, 1 | 8) => TowerEvent::Egg,
        _ if matches!(roll, 2 | 9) => TowerEvent::Enemy,
        _ if floor.floor < highest_floor && room_number > 1 => TowerEvent::Exit,
        _ => TowerEvent::Loot,
    }
}

fn loot_gain(
    floor: &TowerFloorDefinition,
    seed: u64,
    goal: TowerRunGoal,
    pressure_tier: u32,
    profile: &TowerPartyProfile,
) -> Option<ResourceAmount> {
    let base = floor.loot.get((seed as usize) % floor.loot.len().max(1))?;
    let goal_bonus = match goal {
        TowerRunGoal::Salvage => 3,
        TowerRunGoal::SafeRun => -1,
        TowerRunGoal::EggHunt => -1,
        _ => 0,
    };
    let profile_bonus = (profile.salvage / 2) as i32;
    let pressure_bonus = pressure_tier.min(2) as i32;
    let bonus = ((seed >> 8) % 3) as i32 + goal_bonus + profile_bonus + pressure_bonus;
    Some(ResourceAmount {
        resource_id: base.resource_id.clone(),
        amount: (base.amount + bonus).max(1),
    })
}

fn found_egg(
    floor: &TowerFloorDefinition,
    data: &GameData,
    seed: u64,
    goal: TowerRunGoal,
    pressure_tier: u32,
    profile: &TowerPartyProfile,
) -> Option<TowerFoundEgg> {
    let bias = match goal {
        TowerRunGoal::EggHunt => 2,
        TowerRunGoal::Salvage => 0,
        _ => 1,
    } + pressure_tier.min(2)
        + profile.egg_sense;
    let index = ((seed as u32 + bias) as usize) % floor.egg_types.len().max(1);
    let egg_type_id = floor.egg_types.get(index)?;
    let egg_type = data.egg_type(egg_type_id)?;

    Some(TowerFoundEgg {
        egg_type_id: egg_type.id.clone(),
        hatch_days: egg_type.hatch_days,
        origin_floor: floor.floor,
        palette_seed: 0x7000_0000 ^ seed ^ (u64::from(floor.floor) << 24),
    })
}

fn pressure_gain(goal: TowerRunGoal, room_number: u32, state: &GameState, data: &GameData) -> u32 {
    let profile = party_profile(state, data);
    let mut gain = match goal {
        TowerRunGoal::PushDeeper => 2,
        TowerRunGoal::SafeRun if room_number % 2 == 0 => 0,
        TowerRunGoal::Scout if room_number % 3 == 0 => 0,
        _ => 1,
    };
    if profile.safety >= 3 && gain > 0 {
        gain -= 1;
    }
    gain
}

fn enemy_pressure_gain(goal: TowerRunGoal, floor: u32, profile: &TowerPartyProfile) -> u32 {
    let mut gain = (2 + floor / 5).min(4);
    if goal == TowerRunGoal::PushDeeper {
        gain += 1;
    }
    if goal == TowerRunGoal::SafeRun && gain > 1 {
        gain -= 1;
    }
    if profile.safety >= 2 && gain > 1 {
        gain -= 1;
    }
    gain
}

fn pressure_tier(pressure: u32, limit: u32) -> u32 {
    if limit == 0 {
        return 0;
    }
    let scaled = pressure.saturating_mul(9) / limit;
    match scaled {
        0..=2 => 0,
        3..=5 => 1,
        6..=8 => 2,
        _ => 3,
    }
}

fn pressure_flavor(pressure: u32, limit: u32) -> Option<&'static str> {
    match pressure_tier(pressure, limit) {
        1 if pressure > 0 => {
            Some("Pressure hums through the halls: caches improve, patrols tighten.")
        }
        2 => Some("Rare nests stir under high pressure, but enemies hit harder."),
        3 => Some("Pressure is maxed. The next mistake becomes a panic escape."),
        _ => None,
    }
}

#[derive(Default)]
struct TowerPartyProfile {
    egg_sense: u32,
    salvage: u32,
    safety: u32,
}

fn party_profile(state: &GameState, data: &GameData) -> TowerPartyProfile {
    let mut profile = TowerPartyProfile::default();
    for monster_id in state.monster_roster.party_slots.iter().flatten() {
        let Some(monster) = state.monster_roster.monster(*monster_id) else {
            continue;
        };
        let role = data
            .species(&monster.species_id)
            .map(|species| species.role)
            .unwrap_or(monster.role);
        match role {
            MonsterRole::Scout => profile.egg_sense += 1,
            MonsterRole::Tank => profile.safety += 1,
            MonsterRole::Support => profile.safety += 1,
            MonsterRole::Striker => profile.salvage += 1,
        }
        match monster.passive {
            PassiveSkill::FindsSmallLoot | PassiveSkill::FindsStone => profile.salvage += 2,
            PassiveSkill::DetectsEggs => profile.egg_sense += 2,
            PassiveSkill::BurnsBrambles => profile.salvage += 1,
            PassiveSkill::ResistsPoison | PassiveSkill::SoothesInjuries => profile.safety += 2,
        }
    }
    profile
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

fn result(summary: impl Into<String>) -> TowerResult {
    TowerResult {
        summary: summary.into(),
        encounter: None,
    }
}

enum TowerEvent {
    Loot,
    Egg,
    Enemy,
    Exit,
    Boss,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameDataLoader;

    #[test]
    fn return_to_town_respects_hatchery_egg_capacity() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);
        state.town.set_building_level("hatchery", 1);
        state
            .egg_inventory
            .add_egg("mossy_egg".to_owned(), 1, 1, 0x101);
        state
            .egg_inventory
            .add_egg("mossy_egg".to_owned(), 1, 1, 0x102);

        let mut run = TowerRunState::new(1, 9, TowerRunGoal::EggHunt);
        run.found_eggs.push(TowerFoundEgg {
            egg_type_id: "mossy_egg".to_owned(),
            hatch_days: 1,
            origin_floor: 1,
            palette_seed: 0x201,
        });
        run.found_eggs.push(TowerFoundEgg {
            egg_type_id: "mossy_egg".to_owned(),
            hatch_days: 1,
            origin_floor: 1,
            palette_seed: 0x202,
        });
        state.tower_run = Some(run);

        let result = return_to_town(&mut state, &data);

        assert_eq!(state.egg_inventory.eggs.len(), 3);
        assert!(result.summary.contains("1 egg(s)"));
        assert!(result.summary.contains("left 1 egg(s) behind"));
    }
}
