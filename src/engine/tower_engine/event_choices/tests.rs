use super::*;
use crate::data::GameDataLoader;
use crate::engine::tower_engine::start_run;
use crate::state::{GameState, TowerPendingEvent, TowerRunGoal};

#[test]
fn only_successfully_resolved_landmark_approaches_enter_the_guide() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_run(&mut state, &data, TowerRunGoal::Balanced);
    state.tower_run.as_mut().unwrap().pending_event = Some(TowerPendingEvent {
        special_location_id: "lantern_well".to_owned(),
        event_ids: vec!["restorative_draught".to_owned()],
        x: 0,
        y: 0,
    });

    let blocked = choose_special_event(&mut state, &data, "restorative_draught");
    assert!(blocked.summary.contains("needs a party monster"));
    assert!(!state
        .tower_discoveries
        .event_ids
        .contains(&"restorative_draught".to_owned()));

    state.tower_run.as_mut().unwrap().pending_event = Some(TowerPendingEvent {
        special_location_id: "mossbound_shrine".to_owned(),
        event_ids: vec!["moss_mite_offering".to_owned()],
        x: 0,
        y: 0,
    });
    let survey_before = state.tower_run.as_ref().unwrap().survey_charges;
    let first = choose_special_event(&mut state, &data, "moss_mite_offering");

    assert_eq!(state.tower_discoveries.event_ids, ["moss_mite_offering"]);
    assert!(first.summary.contains("1 bonus survey flare"));
    assert_eq!(
        state.tower_run.as_ref().unwrap().survey_charges,
        survey_before + 1
    );
    let completed = &state.tower_run.as_ref().unwrap().completed_landmarks;
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].special_location_id, "mossbound_shrine");
    assert_eq!(completed[0].event_id, "moss_mite_offering");
    assert!(!completed[0].changed_room);

    state.tower_run.as_mut().unwrap().pending_event = Some(TowerPendingEvent {
        special_location_id: "mossbound_shrine".to_owned(),
        event_ids: vec!["moss_mite_offering".to_owned()],
        x: 0,
        y: 0,
    });
    let repeat = choose_special_event(&mut state, &data, "moss_mite_offering");
    assert!(!repeat.summary.contains("bonus survey flare"));
    assert_eq!(
        state.tower_run.as_ref().unwrap().survey_charges,
        survey_before + 1
    );
}
