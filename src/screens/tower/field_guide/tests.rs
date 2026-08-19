use super::*;
use crate::data::GameDataLoader;

#[test]
fn guide_contains_only_persistently_discovered_records() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    assert!(entries(&state, &data).is_empty());

    state.tower_discoveries.discover_enemy("moss_mite");
    state
        .tower_discoveries
        .discover_special_location("mossbound_shrine");
    state.tower_discoveries.discover_hazard("spore_choke");
    let records = entries(&state, &data);

    assert_eq!(records.len(), 3);
    assert!(matches!(records[0], GuideEntry::Enemy(_)));
    assert!(matches!(records[1], GuideEntry::Location(_)));
    assert!(matches!(records[2], GuideEntry::Hazard(_)));
}

#[test]
fn guide_header_explains_survey_preparation_progress() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    assert_eq!(preparation_label(&state), "Field prep unlocks at 0/12");

    for enemy in data.enemies.iter().take(12) {
        state.tower_discoveries.discover_enemy(&enemy.id);
    }
    assert_eq!(
        preparation_label(&state),
        "Field prep +1 SURVEY · rank 2 at 12/30"
    );

    for enemy in data.enemies.iter().skip(12).take(18) {
        state.tower_discoveries.discover_enemy(&enemy.id);
    }
    assert_eq!(preparation_label(&state), "Field prep +2 SURVEY");
}
