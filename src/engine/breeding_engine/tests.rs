use super::*;
use crate::data::GameDataLoader;
use crate::engine::egg_engine;
use crate::state::GameState;

#[test]
fn breeding_creates_an_inherited_hatchable_egg() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    state.town.set_building_level("breeding_grove", 1);
    state.town.set_building_level("hatchery", 1);
    state.resources.add("herbs", 10);
    state.tower_progress.best_floor = 5;

    let rillfin = data.species("rillfin").expect("rillfin should exist");
    let second_id = state
        .monster_roster
        .add_monster("Ripple".to_owned(), rillfin, 0xBEE5_7001);
    let first_id = 1;
    let first_bond = state.monster_roster.monster(first_id).unwrap().bond;
    let second_bond = state.monster_roster.monster(second_id).unwrap().bond;

    let result = breed_pair(&mut state, &data, first_id, second_id);
    assert!(result.summary.contains("nested"));
    assert_eq!(state.egg_inventory.eggs.len(), 1);
    assert_eq!(
        state.monster_roster.monster(first_id).unwrap().bond,
        first_bond + 1
    );
    assert_eq!(
        state.monster_roster.monster(second_id).unwrap().bond,
        second_bond + 1
    );

    let egg_id = state.egg_inventory.eggs[0].id;
    let inheritance = state.egg_inventory.eggs[0]
        .inheritance
        .clone()
        .expect("bred egg should carry inheritance");
    assert!(inheritance.parent_ids.contains(&first_id));
    assert!(inheritance.parent_ids.contains(&second_id));
    assert!(inheritance.species_options.iter().any(|id| id == "slime"));
    assert!(inheritance.species_options.iter().any(|id| id == "rillfin"));
    assert!(inheritance.lineage_quality >= 2);
    assert!(!inheritance.art_profile.species_hint.is_empty());

    state.egg_inventory.egg_mut(egg_id).unwrap().days_remaining = 0;
    let hatch = egg_engine::hatch_egg(&mut state, &data, egg_id);
    assert!(hatch.summary.contains("hatched"));
    let child = state.monster_roster.monsters.last().unwrap();
    assert!(inheritance.species_options.contains(&child.species_id));
    assert!(inheritance.element_options.contains(&child.element));
    assert!(inheritance.temperament_options.contains(&child.temperament));
    assert!(inheritance.passive_options.contains(&child.passive));
    assert!(child.bond >= inheritance.lineage_quality);
    assert!(!child.art_profile.palette.is_empty());
}

#[test]
fn breeding_respects_hatchery_egg_capacity_before_spending_costs() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    state.town.set_building_level("breeding_grove", 1);
    state.town.set_building_level("hatchery", 1);
    state.resources.add("herbs", 10);

    for seed in 0..3 {
        state
            .egg_inventory
            .add_egg("mossy_egg".to_owned(), 1, 1, 0x300 + seed);
    }
    let rillfin = data.species("rillfin").expect("rillfin should exist");
    let second_id = state
        .monster_roster
        .add_monster("Ripple".to_owned(), rillfin, 0xBEE5_7001);
    let herbs_before = state.resources.amount("herbs");

    let result = breed_pair(&mut state, &data, 1, second_id);

    assert!(result.summary.contains("Hatchery egg capacity is full"));
    assert_eq!(state.resources.amount("herbs"), herbs_before);
    assert_eq!(state.egg_inventory.eggs.len(), 3);
}
