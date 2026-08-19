use super::*;
use crate::data::GameDataLoader;
use crate::engine::tower_engine::start_run;
use crate::state::{GameState, TowerMapObjectKind, TowerRunGoal};

#[test]
fn high_pressure_adds_one_named_wandering_enemy() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Balanced);
    let run = state.tower_run.as_mut().expect("run should start");
    let enemy_count = run
        .map
        .objects
        .iter()
        .filter(|object| object.kind == TowerMapObjectKind::Enemy)
        .count();
    run.pressure = run.pressure_limit.saturating_mul(2).div_ceil(3);

    let summary = refresh_pressure(&mut state, &data).expect("threshold should produce a warning");
    let run = state.tower_run.as_ref().unwrap();
    let new_enemy_count = run
        .map
        .objects
        .iter()
        .filter(|object| object.kind == TowerMapObjectKind::Enemy)
        .count();

    assert_eq!(run.pressure_stage, 2);
    assert_eq!(new_enemy_count, enemy_count + 1);
    assert!(summary.contains("wandering"));
    assert!(run
        .map
        .objects
        .iter()
        .any(|object| object.kind == TowerMapObjectKind::Enemy && object.wandering));
    assert!(
        run.map
            .objects
            .iter()
            .filter(|object| {
                object.kind == TowerMapObjectKind::Enemy && data.enemy(&object.enemy_id).is_some()
            })
            .count()
            >= new_enemy_count
    );
    let object_count = run.map.objects.len();

    assert!(refresh_pressure(&mut state, &data).is_none());
    assert_eq!(
        state.tower_run.as_ref().unwrap().map.objects.len(),
        object_count
    );
}

#[test]
fn jumping_to_the_limit_records_every_crossed_wake_stage() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Scout);
    let run = state.tower_run.as_mut().expect("run should start");
    run.pressure = run.pressure_limit;

    let summary = refresh_pressure(&mut state, &data).expect("limit should produce warnings");
    let run = state.tower_run.as_ref().unwrap();

    assert_eq!(run.pressure_stage, 3);
    assert!(summary.contains("tower stirs"));
    assert!(summary.contains("High pressure"));
    assert!(summary.contains("fully awake"));
    assert_eq!(run.event_log.last(), Some(&summary));
}
