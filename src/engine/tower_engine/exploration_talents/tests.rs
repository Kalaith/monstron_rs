use super::*;
use crate::state::{TowerMapObject, TowerRoom};

#[test]
fn entering_a_secret_room_exposes_its_cache_without_revealing_others() {
    let mut map = TowerMapState::new(20, 10, 2, 41);
    map.player_x = 3;
    map.player_y = 3;
    map.rooms = vec![
        TowerRoom {
            start_x: 1,
            start_y: 1,
            width: 5,
            height: 5,
        },
        TowerRoom {
            start_x: 10,
            start_y: 1,
            width: 5,
            height: 5,
        },
    ];
    map.objects = vec![secret_at(4, 4), secret_at(12, 3)];

    assert_eq!(reveal_secret_in_current_room(&mut map), 1);
    assert!(map.objects[0].revealed);
    assert!(!map.objects[1].revealed);
    assert_eq!(reveal_secret_in_current_room(&mut map), 0);
}

fn secret_at(x: u32, y: u32) -> TowerMapObject {
    TowerMapObject {
        kind: TowerMapObjectKind::SecretCache,
        x,
        y,
        resource_id: "coin".to_owned(),
        amount: 2,
        egg_type_id: String::new(),
        hatch_days: 0,
        palette_seed: 0,
        enemy_id: String::new(),
        special_location_id: String::new(),
        event_id: String::new(),
        hazard_id: String::new(),
        wandering: false,
        revealed: false,
    }
}
