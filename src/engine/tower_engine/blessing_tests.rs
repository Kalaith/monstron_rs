use super::*;
use crate::data::GameDataLoader;

#[test]
fn cache_sense_increases_every_cache_pickup() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Salvage);
    state
        .tower_run
        .as_mut()
        .unwrap()
        .add_blessing(TowerBlessing::CacheSense);
    let cache = TowerMapObject {
        kind: TowerMapObjectKind::Loot,
        x: 0,
        y: 0,
        resource_id: "wood".to_owned(),
        amount: 3,
        egg_type_id: String::new(),
        hatch_days: 0,
        palette_seed: 0,
        enemy_id: String::new(),
        special_location_id: String::new(),
        event_id: String::new(),
        hazard_id: String::new(),
        wandering: false,
        revealed: false,
    };

    let result = resolve_map_object(&mut state, &data, cache);

    assert!(result.summary.contains("Found 5"));
    assert_eq!(state.tower_run.as_ref().unwrap().cargo_amount(), 5);
}

#[test]
fn quiet_steps_skips_the_normal_fourth_step_pressure_tick() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Balanced);
    let run = state.tower_run.as_mut().unwrap();
    run.add_blessing(TowerBlessing::QuietSteps);
    run.rooms_explored = 3;
    run.pressure = 0;
    let (dx, dy, target_x, target_y) = adjacent_step(&run.map);
    run.map
        .objects
        .retain(|object| object.x != target_x || object.y != target_y);

    move_party(&mut state, &data, dx, dy);

    assert_eq!(state.tower_run.as_ref().unwrap().rooms_explored, 4);
    assert_eq!(state.tower_run.as_ref().unwrap().pressure, 0);
}

fn adjacent_step(map: &crate::state::TowerMapState) -> (i32, i32, u32, u32) {
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .find_map(|(dx, dy)| {
            let x = map.player_x as i32 + dx;
            let y = map.player_y as i32 + dy;
            (x >= 0 && y >= 0 && map.is_passable(x as u32, y as u32))
                .then_some((dx, dy, x as u32, y as u32))
        })
        .expect("start room should have an adjacent passable tile")
}
