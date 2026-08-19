use super::discovery::record_visible_discoveries;
use super::map_gen::{generate_map, reveal_current_area};
use super::navigation::explore_direction;
use super::*;
use crate::data::GameDataLoader;
use crate::state::{TowerMapState, TowerTileKind, TowerTileVisibility};

#[test]
fn generated_map_has_start_and_stairs() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let state = GameState::new(&data);
    let map = generate_map(&state, &data, 1, TowerRunGoal::Scout, 42);

    assert!(map.is_passable(map.player_x, map.player_y));
    assert!(map
        .objects
        .iter()
        .any(|object| object.kind == TowerMapObjectKind::Stairs));
    assert!(map.rooms.len() >= 4);
    assert!(map.is_visible(map.player_x, map.player_y));
    assert!(map.visibility.contains(&TowerTileVisibility::Hidden));
    assert!(map.objects.iter().any(|object| {
        object.kind == TowerMapObjectKind::SpecialLocation
            && !object.special_location_id.is_empty()
            && !object.event_id.is_empty()
    }));
    assert!(map
        .objects
        .iter()
        .filter(|object| object.kind == TowerMapObjectKind::Enemy)
        .all(|object| data.enemy(&object.enemy_id).is_some()));
    assert!(map
        .objects
        .iter()
        .filter(|object| object.kind == TowerMapObjectKind::Hazard)
        .all(|object| data.tower_hazard(&object.hazard_id).is_some()));
}

#[test]
fn visible_map_content_enters_the_persistent_field_guide_once() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Scout);
    let positions = state
        .tower_run
        .as_ref()
        .unwrap()
        .map
        .objects
        .iter()
        .map(|object| (object.x, object.y))
        .collect::<Vec<_>>();
    for (x, y) in positions {
        state
            .tower_run
            .as_mut()
            .unwrap()
            .map
            .set_visibility(x, y, TowerTileVisibility::Visible);
    }

    record_visible_discoveries(&mut state, &data, None);
    let counts = (
        state.tower_discoveries.enemy_ids.len(),
        state.tower_discoveries.special_location_ids.len(),
        state.tower_discoveries.hazard_ids.len(),
    );
    record_visible_discoveries(&mut state, &data, None);

    assert!(counts.0 > 0 && counts.1 > 0 && counts.2 > 0);
    assert_eq!(
        counts,
        (
            state.tower_discoveries.enemy_ids.len(),
            state.tower_discoveries.special_location_ids.len(),
            state.tower_discoveries.hazard_ids.len(),
        )
    );
}

#[test]
fn special_location_event_applies_its_data_driven_outcome() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Scout);

    let result = resolve_map_object(
        &mut state,
        &data,
        TowerMapObject {
            kind: TowerMapObjectKind::SpecialLocation,
            x: 0,
            y: 0,
            resource_id: String::new(),
            amount: 0,
            egg_type_id: String::new(),
            hatch_days: 0,
            palette_seed: 0,
            enemy_id: String::new(),
            special_location_id: "root_oracle".to_owned(),
            event_id: "roots_reveal_cache".to_owned(),
            hazard_id: String::new(),
        },
    );
    assert!(result.summary.contains("Root Oracle"));
    assert!(state.tower_run.as_ref().unwrap().pending_event.is_some());
    assert_eq!(state.tower_run.as_ref().unwrap().cargo_amount(), 0);

    let result = choose_special_event(&mut state, &data, "roots_reveal_cache");
    let run = state.tower_run.as_ref().expect("tower run should remain");
    assert!(result.summary.contains("Buried Route"));
    assert!(run.pending_event.is_none());
    assert_eq!(
        run.cargo
            .iter()
            .find(|stack| stack.resource_id == "wood")
            .map(|stack| stack.amount),
        Some(5)
    );
    assert_eq!(
        run.cargo
            .iter()
            .find(|stack| stack.resource_id == "herbs")
            .map(|stack| stack.amount),
        Some(3)
    );
}

#[test]
fn party_traits_counter_authored_hazards_and_recover_materials() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Balanced);
    let hp_before = state.monster_roster.monsters[0].hp;

    let result = resolve_hazard(&mut state, &data, "cinder_vent");
    let run = state.tower_run.as_ref().unwrap();

    assert!(result.summary.contains("counters Cinder Vent"));
    assert_eq!(state.monster_roster.monsters[0].hp, hp_before);
    assert_eq!(run.pressure, 0);
    assert_eq!(
        run.cargo
            .iter()
            .find(|stack| stack.resource_id == "ore")
            .map(|stack| stack.amount),
        Some(2)
    );
}

#[test]
fn uncountered_hazard_damages_the_party_and_raises_pressure() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Balanced);
    let hp_before = state.monster_roster.monsters[0].hp;

    let result = resolve_hazard(&mut state, &data, "frostfall");
    let run = state.tower_run.as_ref().unwrap();

    assert!(result.summary.contains("Frostfall Arch catches"));
    assert_eq!(state.monster_roster.monsters[0].hp, hp_before - 4);
    assert_eq!(run.pressure, 2);
}

#[test]
fn landmark_ambush_waits_for_the_players_visible_choice() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Scout);
    state.tower_run.as_mut().unwrap().pending_event = Some(TowerPendingEvent {
        special_location_id: "mossbound_shrine".to_owned(),
        event_ids: vec![
            "keepers_blessing".to_owned(),
            "moss_mite_offering".to_owned(),
        ],
    });

    let result = choose_special_event(&mut state, &data, "moss_mite_offering");

    assert_eq!(
        result.encounter.unwrap().enemy_id.as_deref(),
        Some("moss_mite")
    );
    assert!(state.tower_run.as_ref().unwrap().pending_event.is_none());
}

#[test]
fn party_can_leave_a_landmark_without_triggering_an_outcome() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Scout);
    state.tower_run.as_mut().unwrap().pending_event = Some(TowerPendingEvent {
        special_location_id: "mossbound_shrine".to_owned(),
        event_ids: vec!["keepers_blessing".to_owned()],
    });

    let result = leave_special_event(&mut state, &data);

    assert!(result.summary.contains("undisturbed"));
    assert!(state.tower_run.as_ref().unwrap().pending_event.is_none());
}

#[test]
fn movement_collects_object_on_destination_tile() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);

    start_run(&mut state, &data, TowerRunGoal::Salvage);
    let run = state.tower_run.as_ref().expect("tower run should start");
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let (dx, dy, target_x, target_y) = directions
        .iter()
        .find_map(|(dx, dy)| {
            let x = run.map.player_x as i32 + dx;
            let y = run.map.player_y as i32 + dy;
            if x >= 0 && y >= 0 && run.map.is_passable(x as u32, y as u32) {
                Some((*dx, *dy, x as u32, y as u32))
            } else {
                None
            }
        })
        .expect("start room should have an adjacent passable tile");

    let run = state.tower_run.as_mut().expect("tower run should exist");
    run.map
        .objects
        .retain(|object| object.x != target_x || object.y != target_y);
    run.map.objects.push(TowerMapObject {
        kind: TowerMapObjectKind::Loot,
        x: target_x,
        y: target_y,
        resource_id: "wood".to_owned(),
        amount: 3,
        egg_type_id: String::new(),
        hatch_days: 0,
        palette_seed: 0,
        enemy_id: String::new(),
        special_location_id: String::new(),
        event_id: String::new(),
        hazard_id: String::new(),
    });

    let result = move_party(&mut state, &data, dx, dy);
    let run = state.tower_run.as_ref().expect("tower run should remain");

    assert!(result.summary.contains("Found 3"));
    assert_eq!((run.map.player_x, run.map.player_y), (target_x, target_y));
    assert_eq!(run.rooms_explored, 1);
    assert!(run.map.object_at(target_x, target_y).is_none());
    assert_eq!(run.cargo_amount(), 3);
}

#[test]
fn movement_builds_pressure_at_a_readable_pace() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Balanced);
    let run = state.tower_run.as_ref().expect("tower run should start");
    let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .find(|(dx, dy)| {
            let x = run.map.player_x as i32 + dx;
            let y = run.map.player_y as i32 + dy;
            x >= 0 && y >= 0 && run.map.is_passable(x as u32, y as u32)
        })
        .expect("start room should have a passable neighbor");
    state.tower_run.as_mut().unwrap().rooms_explored = 3;

    move_party(&mut state, &data, dx, dy);

    assert_eq!(state.tower_run.as_ref().unwrap().pressure, 1);
}

#[test]
fn camp_recovers_the_party_but_has_a_travel_cooldown() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::SafeRun);
    state.monster_roster.monsters[0].hp -= 8;
    state.tower_run.as_mut().unwrap().pressure = 4;

    let first = camp_party(&mut state);
    let hp_after_first = state.monster_roster.monsters[0].hp;
    let second = camp_party(&mut state);

    assert!(first.summary.contains("3 total HP"));
    assert_eq!(state.tower_run.as_ref().unwrap().pressure, 2);
    assert_eq!(state.tower_run.as_ref().unwrap().camp_cooldown, 8);
    assert_eq!(state.monster_roster.monsters[0].hp, hp_after_first);
    assert!(second.summary.contains("travel 8 more step"));
}

#[test]
fn explore_finds_a_step_toward_hidden_passable_space() {
    let mut map = TowerMapState::new(6, 3, 1, 11);
    for x in 1..5 {
        map.set_tile(x, 1, TowerTileKind::Corridor);
    }
    map.player_x = 1;
    map.player_y = 1;
    map.set_visibility(1, 1, TowerTileVisibility::Visible);
    map.set_visibility(2, 1, TowerTileVisibility::Explored);

    assert_eq!(explore_direction(&map), Some((1, 0)));
}

#[test]
fn movement_keeps_only_discovered_tiles_revealed() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);

    start_run(&mut state, &data, TowerRunGoal::Scout);
    let run = state.tower_run.as_ref().expect("tower run should start");
    let hidden_before = run
        .map
        .visibility
        .iter()
        .filter(|visibility| **visibility == TowerTileVisibility::Hidden)
        .count();
    let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .find_map(|(dx, dy)| {
            let x = run.map.player_x as i32 + dx;
            let y = run.map.player_y as i32 + dy;
            if x >= 0 && y >= 0 && run.map.is_passable(x as u32, y as u32) {
                Some((*dx, *dy))
            } else {
                None
            }
        })
        .expect("start room should have an adjacent passable tile");

    move_party(&mut state, &data, dx, dy);
    let run = state.tower_run.as_ref().expect("tower run should remain");
    let hidden_after = run
        .map
        .visibility
        .iter()
        .filter(|visibility| **visibility == TowerTileVisibility::Hidden)
        .count();

    assert!(run.map.is_visible(run.map.player_x, run.map.player_y));
    assert!(hidden_after > 0);
    assert!(hidden_after <= hidden_before);
}

#[test]
fn reveal_current_area_marks_previous_tiles_explored() {
    let mut map = TowerMapState::new(12, 5, 1, 7);
    for x in 1..11 {
        map.set_tile(x, 2, TowerTileKind::Corridor);
    }
    map.player_x = 1;
    map.player_y = 2;
    reveal_current_area(&mut map);

    assert!(map.is_visible(1, 2));

    map.player_x = 10;
    map.player_y = 2;
    reveal_current_area(&mut map);

    assert_eq!(map.visibility_at(1, 2), TowerTileVisibility::Explored);
    assert!(map.is_visible(10, 2));
}

#[test]
fn ensure_map_restores_visibility_for_older_runs() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);

    start_run(&mut state, &data, TowerRunGoal::SafeRun);
    {
        let run = state.tower_run.as_mut().expect("tower run should start");
        run.map.visibility.clear();
    }

    ensure_map(&mut state, &data);
    let run = state.tower_run.as_ref().expect("tower run should remain");

    assert_eq!(
        run.map.visibility.len(),
        (run.map.width * run.map.height) as usize
    );
    assert!(run.map.is_visible(run.map.player_x, run.map.player_y));
    assert!(run
        .event_log
        .iter()
        .any(|event| event.contains("map notes")));
}

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
