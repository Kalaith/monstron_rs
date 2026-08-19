use super::survey::reveal_hidden_room_for_goal;
use super::*;
use crate::data::GameDataLoader;
use crate::state::{TowerRoomKind, TowerTileVisibility};

#[test]
fn salvage_survey_exposes_and_resolves_an_authored_secret_cache() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Salvage);
    let run = state.tower_run.as_mut().unwrap();
    let secret = run
        .map
        .objects
        .iter()
        .find(|object| object.kind == TowerMapObjectKind::SecretCache)
        .cloned()
        .expect("generated floor should contain a secret cache");
    let secret_room = run
        .map
        .rooms
        .iter()
        .position(|room| {
            secret.x >= room.start_x
                && secret.x < room.start_x + room.width
                && secret.y >= room.start_y
                && secret.y < room.start_y + room.height
        })
        .expect("secret should belong to a room");
    run.map.room_kinds.fill(TowerRoomKind::Traversal);
    run.map.room_kinds[secret_room] = TowerRoomKind::Cache;
    run.map.visibility.fill(TowerTileVisibility::Hidden);
    run.map.set_visibility(
        run.map.player_x,
        run.map.player_y,
        TowerTileVisibility::Visible,
    );

    reveal_hidden_room_for_goal(&mut run.map, TowerRunGoal::Salvage)
        .expect("salvage survey should find the cache room");
    let index = run
        .map
        .objects
        .iter()
        .position(|object| object.kind == TowerMapObjectKind::SecretCache)
        .unwrap();
    assert!(run.map.objects[index].revealed);
    let revealed = run.map.objects.remove(index);
    let cargo_before = run.cargo_amount();

    let result = resolve_map_object(&mut state, &data, revealed);

    assert!(result.summary.contains("Opened a surveyed secret"));
    assert!(state.tower_run.as_ref().unwrap().cargo_amount() > cargo_before);
}
