use super::*;
use crate::state::TowerTileKind;

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
