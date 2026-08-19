use super::*;

#[test]
fn field_guide_records_unlock_two_survey_preparation_ranks() {
    let mut discoveries = TowerDiscoveryState::default();
    assert_eq!(discoveries.record_count(), 0);
    assert_eq!(discoveries.survey_bonus(), 0);

    for index in 0..12 {
        discoveries.discover_enemy(&format!("enemy_{index}"));
    }
    assert_eq!(discoveries.record_count(), 12);
    assert_eq!(discoveries.survey_bonus(), 1);

    for index in 0..12 {
        discoveries.discover_special_location(&format!("landmark_{index}"));
    }
    for index in 0..6 {
        discoveries.discover_hazard(&format!("hazard_{index}"));
    }
    assert_eq!(discoveries.record_count(), 30);
    assert_eq!(discoveries.survey_bonus(), 2);

    assert!(!discoveries.discover_enemy("enemy_0"));
    assert_eq!(discoveries.record_count(), 30);

    assert!(discoveries.discover_event("keepers_blessing"));
    assert!(!discoveries.discover_event("keepers_blessing"));
    assert_eq!(discoveries.record_count(), 30);
}
