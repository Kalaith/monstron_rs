use super::*;
use crate::data::GameDataLoader;

#[test]
fn only_recorded_wandering_enemies_leave_tracks_in_explored_rooms() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    let mut map = TowerMapState::new(8, 8, 1, 77);
    let mut hunter = enemy_at("moss_mite", 4, 4);
    map.set_visibility(4, 4, TowerTileVisibility::Explored);

    assert!(!should_draw_known_track(&state, &map, &hunter));
    state.tower_discoveries.discover_enemy("moss_mite");
    assert!(should_draw_known_track(&state, &map, &hunter));

    map.set_visibility(4, 4, TowerTileVisibility::Visible);
    assert!(!should_draw_known_track(&state, &map, &hunter));
    map.set_visibility(4, 4, TowerTileVisibility::Explored);
    hunter.wandering = false;
    assert!(!should_draw_known_track(&state, &map, &hunter));
}

fn enemy_at(enemy_id: &str, x: u32, y: u32) -> TowerMapObject {
    TowerMapObject {
        kind: TowerMapObjectKind::Enemy,
        x,
        y,
        resource_id: String::new(),
        amount: 0,
        egg_type_id: String::new(),
        hatch_days: 0,
        palette_seed: 0,
        enemy_id: enemy_id.to_owned(),
        special_location_id: String::new(),
        event_id: String::new(),
        hazard_id: String::new(),
        wandering: true,
        revealed: false,
    }
}
