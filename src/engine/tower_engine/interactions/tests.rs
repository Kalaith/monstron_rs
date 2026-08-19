use super::*;
use crate::data::GameDataLoader;
use crate::engine::tower_engine::start_run;
use crate::state::{GameState, TowerRunGoal};

#[test]
fn authored_event_grants_a_persistent_run_blessing() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Scout);

    let result = apply_tower_event(&mut state, &data, "mossbound_shrine", "keepers_blessing");

    assert!(result.summary.contains("Gained Quiet Steps"));
    assert!(state
        .tower_run
        .as_ref()
        .unwrap()
        .has_blessing(TowerBlessing::QuietSteps));
}

#[test]
fn authored_event_restocks_survey_and_repels_only_wandering_enemies() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, crate::state::TowerRunGoal::Balanced);
    let run = state.tower_run.as_mut().unwrap();
    run.survey_charges = 0;
    let mut hunter = run
        .map
        .objects
        .iter()
        .find(|object| object.kind == crate::state::TowerMapObjectKind::Enemy)
        .cloned()
        .expect("generated floor should contain a denizen");
    hunter.wandering = true;
    run.map.objects.push(hunter);

    let result = apply_tower_event(
        &mut state,
        &data,
        "whispering_cartographer",
        "chartmakers_signal",
    );
    let run = state.tower_run.as_ref().unwrap();

    assert!(result.summary.contains("Restocked 2 survey flare"));
    assert!(result.summary.contains("Drove off 1 wandering hunter"));
    assert_eq!(run.survey_charges, 2);
    assert_eq!(
        run.map
            .objects
            .iter()
            .filter(|object| object.wandering)
            .count(),
        0
    );
    assert!(run.map.objects.iter().any(|object| !object.wandering));
}

#[test]
fn root_oracle_event_charts_the_authored_secret_cache_room() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Balanced);
    let secret_position = state
        .tower_run
        .as_ref()
        .unwrap()
        .map
        .objects
        .iter()
        .find(|object| object.kind == crate::state::TowerMapObjectKind::SecretCache)
        .map(|object| (object.x, object.y))
        .expect("floor should contain a concealed cache");

    let result = apply_tower_event(&mut state, &data, "root_oracle", "roots_reveal_cache");
    let run = state.tower_run.as_ref().unwrap();
    let secret = run
        .map
        .object_at(secret_position.0, secret_position.1)
        .unwrap();

    assert!(result.summary.contains("Charted 1 concealed cache room"));
    assert!(secret.revealed);
    assert!(run.map.is_visible(secret.x, secret.y));
}

#[test]
fn recovery_event_converts_its_landmark_room_into_a_routable_shelter() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::SafeRun);
    let run = state.tower_run.as_mut().unwrap();
    run.map.ensure_room_kinds();
    run.map.room_kinds[0] = crate::state::TowerRoomKind::Landmark;

    let result = apply_tower_event(&mut state, &data, "lantern_well", "restorative_draught");
    let run = state.tower_run.as_ref().unwrap();

    assert!(result.summary.contains("marked CAMP shelter"));
    assert_eq!(run.map.room_kind(0), crate::state::TowerRoomKind::Camp);
    assert!(crate::engine::tower_engine::camp_sheltered(run));
}

#[test]
fn wardstone_is_consumed_to_cancel_an_uncountered_hazard() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Balanced);
    state
        .tower_run
        .as_mut()
        .unwrap()
        .add_blessing(TowerBlessing::Wardstone);
    let hp_before = state.monster_roster.monsters[0].hp;
    let pressure_before = state.tower_run.as_ref().unwrap().pressure;

    let result = resolve_hazard(&mut state, &data, "frostfall");
    let run = state.tower_run.as_ref().unwrap();

    assert!(result.summary.contains("Wardstone shatters"));
    assert_eq!(state.monster_roster.monsters[0].hp, hp_before);
    assert_eq!(run.pressure, pressure_before);
    assert!(!run.has_blessing(TowerBlessing::Wardstone));
    assert_eq!(run.stats.hazards_countered, 1);
}
