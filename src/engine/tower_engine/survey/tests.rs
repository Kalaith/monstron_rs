use super::*;
use crate::state::{TowerMapState, TowerRoom, TowerTileKind};

#[test]
fn survey_reveals_the_nearest_hidden_room() {
    let mut map = TowerMapState::new(20, 10, 1, 7);
    map.player_x = 2;
    map.player_y = 3;
    map.rooms = vec![
        TowerRoom {
            start_x: 1,
            start_y: 1,
            width: 4,
            height: 4,
        },
        TowerRoom {
            start_x: 7,
            start_y: 1,
            width: 4,
            height: 4,
        },
        TowerRoom {
            start_x: 14,
            start_y: 1,
            width: 4,
            height: 4,
        },
    ];
    for room in map.rooms.clone() {
        for y in room.start_y..room.start_y + room.height {
            for x in room.start_x..room.start_x + room.width {
                map.set_tile(x, y, TowerTileKind::Floor);
            }
        }
    }
    map.set_visibility(3, 3, TowerTileVisibility::Visible);

    let reveal = reveal_hidden_room_for_goal(&mut map, TowerRunGoal::Balanced)
        .expect("a hidden room should remain");

    assert_eq!(reveal.center, (9, 3));
    assert_eq!(map.visibility_at(9, 3), TowerTileVisibility::Visible);
    assert_eq!(map.visibility_at(16, 3), TowerTileVisibility::Hidden);
}

#[test]
fn egg_hunt_survey_prefers_a_far_nest_over_a_near_generic_room() {
    let mut map = TowerMapState::new(20, 10, 1, 9);
    map.player_x = 2;
    map.player_y = 3;
    map.rooms = vec![
        TowerRoom {
            start_x: 1,
            start_y: 1,
            width: 4,
            height: 4,
        },
        TowerRoom {
            start_x: 7,
            start_y: 1,
            width: 4,
            height: 4,
        },
        TowerRoom {
            start_x: 14,
            start_y: 1,
            width: 4,
            height: 4,
        },
    ];
    map.ensure_room_kinds();
    map.set_room_kind(0, TowerRoomKind::Camp);
    map.set_room_kind(1, TowerRoomKind::Traversal);
    map.set_room_kind(2, TowerRoomKind::Nest);
    for room in map.rooms.clone() {
        for y in room.start_y..room.start_y + room.height {
            for x in room.start_x..room.start_x + room.width {
                map.set_tile(x, y, TowerTileKind::Floor);
            }
        }
    }
    map.set_visibility(3, 3, TowerTileVisibility::Visible);

    let reveal =
        reveal_hidden_room_for_goal(&mut map, TowerRunGoal::EggHunt).expect("a nest should remain");

    assert_eq!(reveal.center, (16, 3));
    assert_eq!(map.visibility_at(16, 3), TowerTileVisibility::Visible);
    assert_eq!(map.visibility_at(9, 3), TowerTileVisibility::Hidden);
}
