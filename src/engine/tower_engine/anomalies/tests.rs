use super::*;
use crate::data::GameDataLoader;

#[test]
fn anomaly_selection_is_deterministic_and_floor_eligible() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let first = select_anomaly_id(&data, 5, 0xA110_0045);
    let second = select_anomaly_id(&data, 5, 0xA110_0045);

    assert_eq!(first, second);
    let anomaly = data.tower_anomaly(&first).expect("anomaly should exist");
    assert!(anomaly.min_floor <= 5 && anomaly.max_floor >= 5);
    assert!(anomaly.visual_index < 6);
}
