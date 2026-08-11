use super::*;
use crate::data::GameDataLoader;

#[test]
fn stable_capacity_blocks_hatching_without_consuming_egg() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    state.town.set_building_level("hatchery", 1);

    let rootling = data.species("rootling").expect("rootling should exist");
    let rillfin = data.species("rillfin").expect("rillfin should exist");
    state
        .monster_roster
        .add_monster("Root".to_owned(), rootling, 0x1001);
    state
        .monster_roster
        .add_monster("Ripple".to_owned(), rillfin, 0x1002);
    let egg_id = state
        .egg_inventory
        .add_egg("mossy_egg".to_owned(), 0, 1, 0x2001);

    let result = hatch_egg(&mut state, &data, egg_id);

    assert!(result.summary.contains("Stable capacity is full"));
    assert_eq!(state.monster_roster.monsters.len(), 3);
    assert!(state.egg_inventory.eggs.iter().any(|egg| egg.id == egg_id));
}

#[test]
fn upgraded_stable_allows_hatching_past_base_capacity() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    state.town.set_building_level("hatchery", 1);
    state.town.set_building_level("stable", 1);

    let rootling = data.species("rootling").expect("rootling should exist");
    let rillfin = data.species("rillfin").expect("rillfin should exist");
    state
        .monster_roster
        .add_monster("Root".to_owned(), rootling, 0x1001);
    state
        .monster_roster
        .add_monster("Ripple".to_owned(), rillfin, 0x1002);
    let egg_id = state
        .egg_inventory
        .add_egg("mossy_egg".to_owned(), 0, 1, 0x2001);

    let result = hatch_egg(&mut state, &data, egg_id);

    assert!(result.summary.contains("hatched"));
    assert_eq!(state.monster_roster.monsters.len(), 4);
    assert!(state.egg_inventory.eggs.is_empty());
}
