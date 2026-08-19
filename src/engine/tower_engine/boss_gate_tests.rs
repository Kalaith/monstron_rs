use super::*;
use crate::data::GameDataLoader;

#[test]
fn crown_exit_reseals_until_the_guardian_falls() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::PushDeeper);
    let run = state.tower_run.as_mut().unwrap();
    run.current_floor = 10;
    run.map.floor = 10;
    run.map.objects.clear();
    let exit = TowerMapObject {
        kind: TowerMapObjectKind::Exit,
        x: run.map.player_x,
        y: run.map.player_y,
        resource_id: String::new(),
        amount: 0,
        egg_type_id: String::new(),
        hatch_days: 0,
        palette_seed: 0,
        enemy_id: String::new(),
        special_location_id: String::new(),
        event_id: String::new(),
        hazard_id: String::new(),
        wandering: false,
    };

    let result = resolve_map_object(&mut state, &data, exit);

    assert!(result.summary.contains("threshold is sealed"));
    assert!(!result.returned_to_town);
    let run = state.tower_run.as_ref().expect("sealed run should remain");
    assert_eq!(run.map.objects.len(), 1);
    assert_eq!(run.map.objects[0].kind, TowerMapObjectKind::Exit);
}
