use super::map_gen::{generate_map, reveal_current_area};
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
