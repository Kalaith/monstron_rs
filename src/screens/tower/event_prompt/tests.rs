use super::*;
use crate::data::GameDataLoader;

#[test]
fn event_preview_marks_tried_choices_and_names_advanced_effects() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let event = data
        .tower_event("roots_reveal_cache")
        .expect("root cache event should exist");

    assert_eq!(
        choice_label(0, Some(event), true),
        "1  Buried Route  ·  TRIED"
    );
    let preview = effect_summary(&data, event);
    assert!(preview.contains("Reveal 1 secret"));
    assert!(preview.contains("Gain Cache Sense"));

    let shelter = data
        .tower_event("rekindle_camp")
        .expect("waykeeper shelter event should exist");
    assert!(effect_summary(&data, shelter).contains("Create shelter"));
}
