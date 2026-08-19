use super::*;
use crate::data::GameDataLoader;
use crate::state::{TowerMapState, TowerRoom, TowerRunGoal};

#[test]
fn marked_camp_room_improves_healing_pressure_and_cooldown() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    let mut map = TowerMapState::new(12, 8, 1, 9);
    map.rooms.push(TowerRoom {
        start_x: 1,
        start_y: 1,
        width: 5,
        height: 5,
    });
    map.ensure_room_kinds();
    map.set_room_kind(0, TowerRoomKind::Camp);
    map.player_x = 3;
    map.player_y = 3;
    let mut run = TowerRunState::new(1, 10, TowerRunGoal::Balanced).with_map(map);
    run.pressure = 5;
    state.tower_run = Some(run);
    let party_id = state.monster_roster.party_slots[0].unwrap();
    let monster = state.monster_roster.monster_mut(party_id).unwrap();
    monster.hp = (monster.max_hp - 7).max(1);
    let before = monster.hp;

    let result = camp_party(&mut state, &data);
    let run = state.tower_run.as_ref().unwrap();
    let healed = state.monster_roster.monster(party_id).unwrap().hp - before;

    assert!(result.summary.contains("marked shelter"));
    assert_eq!(healed, 5);
    assert_eq!(run.pressure, 2);
    assert_eq!(run.camp_cooldown, 6);
}

#[test]
fn corridor_camp_retains_the_weaker_fallback() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    let mut map = TowerMapState::new(12, 8, 1, 11);
    map.player_x = 9;
    map.player_y = 6;
    let mut run = TowerRunState::new(1, 10, TowerRunGoal::Balanced).with_map(map);
    run.pressure = 5;
    state.tower_run = Some(run);

    let result = camp_party(&mut state, &data);
    let run = state.tower_run.as_ref().unwrap();

    assert!(result.summary.contains("corridor camp"));
    assert_eq!(run.pressure, 3);
    assert_eq!(run.camp_cooldown, 8);
}
