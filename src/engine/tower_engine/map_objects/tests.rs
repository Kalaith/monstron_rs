use super::*;
use crate::data::GameDataLoader;
use crate::state::TowerTileKind;
use std::collections::BTreeSet;

#[test]
fn floor_roster_draws_each_eligible_species_before_repeating() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let eligible = eligible_enemies(&data, 5, false);
    assert!(eligible.len() >= 8);
    let mut rng = TowerMapRng::new(155);

    let selected = varied_enemy_roster(eligible, 8, &mut rng);
    let ids = selected
        .iter()
        .map(|enemy| enemy.id.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(selected.len(), 8);
    assert_eq!(ids.len(), 8);
}

#[test]
fn pressure_hunter_prefers_a_species_not_already_on_the_floor() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let eligible = eligible_enemies(&data, 1, false);
    let fresh_id = eligible.last().unwrap().id.clone();
    let mut map = TowerMapState::new(18, 10, 1, 201);
    map.rooms.push(crate::state::TowerRoom {
        start_x: 1,
        start_y: 1,
        width: 16,
        height: 8,
    });
    for y in 1..9 {
        for x in 1..17 {
            map.set_tile(x, y, crate::state::TowerTileKind::Floor);
        }
    }
    for enemy in eligible.iter().take(eligible.len() - 1) {
        map.objects.push(TowerMapObject::enemy(&enemy.id));
    }

    let spawned = spawn_pressure_enemy(&mut map, &data, 19);

    assert_eq!(spawned.as_deref(), Some(fresh_id.as_str()));
}

#[test]
fn wandering_enemy_routes_around_an_occupied_tile() {
    let mut map = open_chase_map();
    map.player_x = 5;
    map.player_y = 2;
    let mut hunter = TowerMapObject::enemy("moss_crawler");
    hunter.x = 1;
    hunter.y = 2;
    hunter.wandering = true;
    let mut blocker = TowerMapObject::hazard("spore_bloom");
    blocker.x = 2;
    blocker.y = 2;
    map.objects = vec![hunter, blocker];

    assert!(matches!(
        advance_wandering_enemy(&mut map),
        Some(WanderingAdvance::Moved)
    ));
    let hunter = map.objects.iter().find(|object| object.wandering).unwrap();
    assert_eq!((hunter.x, hunter.y), (1, 1));
}

#[test]
fn wandering_enemy_intercepts_and_leaves_the_map() {
    let mut map = open_chase_map();
    map.player_x = 5;
    map.player_y = 2;
    let mut hunter = TowerMapObject::enemy("moss_crawler");
    hunter.x = 4;
    hunter.y = 2;
    hunter.wandering = true;
    map.objects.push(hunter);

    let Some(WanderingAdvance::Encounter(encounter)) = advance_wandering_enemy(&mut map) else {
        panic!("adjacent hunter should intercept the party");
    };
    assert_eq!(encounter.enemy_id, "moss_crawler");
    assert!(encounter.wandering);
    assert!(map.objects.is_empty());
}

fn open_chase_map() -> TowerMapState {
    let mut map = TowerMapState::new(7, 5, 1, 41);
    for y in 1..4 {
        for x in 1..6 {
            map.set_tile(x, y, TowerTileKind::Corridor);
        }
    }
    map
}
