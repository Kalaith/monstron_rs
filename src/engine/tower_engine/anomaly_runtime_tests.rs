use super::*;
use crate::data::GameDataLoader;

#[test]
fn echo_rain_raises_pressure_on_the_third_step() {
    let (data, mut state) = run_with_anomaly("echo_rain");
    let run = state.tower_run.as_mut().unwrap();
    run.rooms_explored = 2;
    run.pressure = 0;
    let (dx, dy, x, y) = adjacent_step(&run.map);
    run.map
        .objects
        .retain(|object| object.x != x || object.y != y);

    move_party(&mut state, &data, dx, dy);

    assert_eq!(state.tower_run.as_ref().unwrap().pressure, 1);
}

#[test]
fn veil_mist_skips_the_normal_fourth_step_pressure_tick() {
    let (data, mut state) = run_with_anomaly("veil_mist");
    let run = state.tower_run.as_mut().unwrap();
    run.rooms_explored = 3;
    run.pressure = 0;
    let (dx, dy, x, y) = adjacent_step(&run.map);
    run.map
        .objects
        .retain(|object| object.x != x || object.y != y);

    move_party(&mut state, &data, dx, dy);

    assert_eq!(state.tower_run.as_ref().unwrap().pressure, 0);
}

#[test]
fn crystal_bloom_adds_material_to_cache_pickups() {
    let (data, mut state) = run_with_anomaly("crystal_bloom");
    let cache = map_object(TowerMapObjectKind::Loot, "wood", 3, 0);

    resolve_map_object(&mut state, &data, cache);

    assert_eq!(state.tower_run.as_ref().unwrap().cargo_amount(), 5);
}

#[test]
fn nesting_pollen_shortens_found_egg_incubation() {
    let (data, mut state) = run_with_anomaly("nesting_pollen");
    let mut egg = map_object(TowerMapObjectKind::Egg, "", 0, 4);
    egg.egg_type_id = "mossy_egg".to_owned();

    resolve_map_object(&mut state, &data, egg);

    assert_eq!(
        state.tower_run.as_ref().unwrap().found_eggs[0].hatch_days,
        3
    );
}

#[test]
fn mending_lights_improve_camp_healing_and_cooldown() {
    let (data, mut state) = run_with_anomaly("mending_lights");
    state.monster_roster.monsters[0].hp -= 8;
    let hp_before = state.monster_roster.monsters[0].hp;

    camp_party(&mut state, &data);

    assert_eq!(state.monster_roster.monsters[0].hp, hp_before + 5);
    assert_eq!(state.tower_run.as_ref().unwrap().camp_cooldown, 6);
}

#[test]
fn hunter_tracks_move_a_wandering_enemy_every_step() {
    let (data, mut state) = run_with_anomaly("hunter_tracks");
    let mut map = crate::state::TowerMapState::new(7, 5, 5, 51);
    for y in 1..4 {
        for x in 1..6 {
            map.set_tile(x, y, crate::state::TowerTileKind::Corridor);
        }
    }
    map.player_x = 3;
    map.player_y = 2;
    let mut hunter = map_object(TowerMapObjectKind::Enemy, "", 0, 0);
    hunter.enemy_id = "crystal_anemone".to_owned();
    hunter.x = 1;
    hunter.y = 2;
    hunter.wandering = true;
    map.objects.push(hunter);
    state.tower_run.as_mut().unwrap().map = map;

    move_party(&mut state, &data, 1, 0);

    let hunter = state
        .tower_run
        .as_ref()
        .unwrap()
        .map
        .objects
        .iter()
        .find(|object| object.wandering)
        .expect("hunter should remain on the map");
    assert_eq!((hunter.x, hunter.y), (2, 2));
}

fn run_with_anomaly(id: &str) -> (GameData, GameState) {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Balanced);
    state.tower_run.as_mut().unwrap().anomaly_id = id.to_owned();
    (data, state)
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

fn map_object(
    kind: TowerMapObjectKind,
    resource_id: &str,
    amount: i32,
    hatch_days: u32,
) -> TowerMapObject {
    TowerMapObject {
        kind,
        x: 0,
        y: 0,
        resource_id: resource_id.to_owned(),
        amount,
        egg_type_id: String::new(),
        hatch_days,
        palette_seed: 17,
        enemy_id: String::new(),
        special_location_id: String::new(),
        event_id: String::new(),
        hazard_id: String::new(),
        wandering: false,
        revealed: false,
    }
}
