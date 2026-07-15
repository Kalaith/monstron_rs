use super::map_objects::add_map_objects;
use crate::data::GameData;
use crate::state::{
    GameState, TowerMapRng, TowerMapState, TowerRoom, TowerRunGoal, TowerTileKind,
    TowerTileVisibility,
};

pub(super) fn generate_map(
    state: &GameState,
    data: &GameData,
    floor_number: u32,
    goal: TowerRunGoal,
    seed: u64,
) -> TowerMapState {
    let mut rng = TowerMapRng::new(seed);
    let width = 30 + (floor_number / 4).min(2) * 2;
    let height = 22;
    let mut map = TowerMapState::new(width, height, floor_number, seed);
    let room_target = room_target(floor_number, goal);

    for _ in 0..room_target * 12 {
        if map.rooms.len() >= room_target as usize {
            break;
        }
        let room_width = rng.range(5, 9);
        let room_height = rng.range(4, 7);
        let room = TowerRoom {
            width: room_width,
            height: room_height,
            start_x: rng.range(1, width.saturating_sub(room_width + 1).max(2)),
            start_y: rng.range(1, height.saturating_sub(room_height + 1).max(2)),
        };

        if room.start_x + room.width >= width - 1 || room.start_y + room.height >= height - 1 {
            continue;
        }
        if map
            .rooms
            .iter()
            .any(|existing| room.intersects_padded(*existing))
        {
            continue;
        }

        carve_room(&mut map, room);
        if let Some(previous) = map.rooms.last().copied() {
            carve_corridor(&mut map, previous.center(), room.center(), &mut rng);
        }
        map.rooms.push(room);
    }

    if map.rooms.len() < 4 {
        carve_fallback_layout(&mut map, &mut rng);
    }

    let start_room = map.rooms[0];
    let (start_x, start_y) = start_room.random_inner(&mut rng);
    map.start_x = start_x;
    map.start_y = start_y;
    map.player_x = start_x;
    map.player_y = start_y;

    add_map_objects(&mut map, state, data, floor_number, goal, &mut rng);
    reveal_current_area(&mut map);
    map
}

fn room_target(floor: u32, goal: TowerRunGoal) -> u32 {
    let base = 8 + (floor / 2).min(5);
    match goal {
        TowerRunGoal::Scout => base + 2,
        TowerRunGoal::PushDeeper => base + 1,
        TowerRunGoal::SafeRun => base.saturating_sub(1),
        _ => base,
    }
}

fn carve_room(map: &mut TowerMapState, room: TowerRoom) {
    for x in room.start_x + 1..room.start_x + room.width - 1 {
        for y in room.start_y + 1..room.start_y + room.height - 1 {
            map.set_tile(x, y, TowerTileKind::Floor);
        }
    }
}

fn carve_corridor(
    map: &mut TowerMapState,
    start: (u32, u32),
    end: (u32, u32),
    rng: &mut TowerMapRng,
) {
    if rng.chance(1, 2) {
        carve_horizontal(map, start.0, end.0, start.1);
        carve_vertical(map, start.1, end.1, end.0);
    } else {
        carve_vertical(map, start.1, end.1, start.0);
        carve_horizontal(map, start.0, end.0, end.1);
    }
}

fn carve_fallback_layout(map: &mut TowerMapState, rng: &mut TowerMapRng) {
    map.tiles.fill(TowerTileKind::Wall);
    map.rooms.clear();

    let width = map.width;
    let height = map.height;
    let rooms = [
        TowerRoom {
            start_x: 2,
            start_y: 2,
            width: 7,
            height: 6,
        },
        TowerRoom {
            start_x: width / 2 - 4,
            start_y: 2,
            width: 8,
            height: 6,
        },
        TowerRoom {
            start_x: width - 10,
            start_y: 3,
            width: 8,
            height: 6,
        },
        TowerRoom {
            start_x: 3,
            start_y: height - 9,
            width: 8,
            height: 6,
        },
        TowerRoom {
            start_x: width / 2 - 3,
            start_y: height - 8,
            width: 7,
            height: 6,
        },
        TowerRoom {
            start_x: width - 11,
            start_y: height - 9,
            width: 9,
            height: 6,
        },
    ];

    let mut previous_center = None;
    for room in rooms {
        carve_room(map, room);
        if let Some(previous) = previous_center {
            carve_corridor(map, previous, room.center(), rng);
        }
        previous_center = Some(room.center());
        map.rooms.push(room);
    }
}

fn carve_horizontal(map: &mut TowerMapState, from_x: u32, to_x: u32, y: u32) {
    let start = from_x.min(to_x);
    let end = from_x.max(to_x);
    for x in start..=end {
        carve_corridor_tile(map, x, y);
    }
}

fn carve_vertical(map: &mut TowerMapState, from_y: u32, to_y: u32, x: u32) {
    let start = from_y.min(to_y);
    let end = from_y.max(to_y);
    for y in start..=end {
        carve_corridor_tile(map, x, y);
    }
}

fn carve_corridor_tile(map: &mut TowerMapState, x: u32, y: u32) {
    if map.tile_at(x, y) == TowerTileKind::Wall {
        map.set_tile(x, y, TowerTileKind::Corridor);
    }
}

pub(super) fn reveal_current_area(map: &mut TowerMapState) {
    map.ensure_visibility();
    for visibility in &mut map.visibility {
        if *visibility == TowerTileVisibility::Visible {
            *visibility = TowerTileVisibility::Explored;
        }
    }

    let player_x = map.player_x;
    let player_y = map.player_y;
    if let Some(room) = room_containing(map, player_x, player_y) {
        reveal_room(map, room);
        reveal_radius(map, player_x, player_y, 2);
        reveal_room_edges(map, room);
    } else {
        reveal_radius(map, player_x, player_y, 3);
    }
}

fn room_containing(map: &TowerMapState, x: u32, y: u32) -> Option<TowerRoom> {
    map.rooms.iter().copied().find(|room| {
        x >= room.start_x
            && x < room.start_x + room.width
            && y >= room.start_y
            && y < room.start_y + room.height
    })
}

fn reveal_room(map: &mut TowerMapState, room: TowerRoom) {
    let max_x = (room.start_x + room.width).min(map.width);
    let max_y = (room.start_y + room.height).min(map.height);
    for y in room.start_y..max_y {
        for x in room.start_x..max_x {
            map.set_visibility(x, y, TowerTileVisibility::Visible);
        }
    }
}

fn reveal_room_edges(map: &mut TowerMapState, room: TowerRoom) {
    let min_x = room.start_x.saturating_sub(1);
    let min_y = room.start_y.saturating_sub(1);
    let max_x = (room.start_x + room.width).min(map.width.saturating_sub(1));
    let max_y = (room.start_y + room.height).min(map.height.saturating_sub(1));

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if map.is_passable(x, y) {
                map.set_visibility(x, y, TowerTileVisibility::Visible);
            }
        }
    }
}

fn reveal_radius(map: &mut TowerMapState, center_x: u32, center_y: u32, radius: u32) {
    let min_x = center_x.saturating_sub(radius);
    let min_y = center_y.saturating_sub(radius);
    let max_x = (center_x + radius).min(map.width.saturating_sub(1));
    let max_y = (center_y + radius).min(map.height.saturating_sub(1));

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let distance = center_x.abs_diff(x) + center_y.abs_diff(y);
            if distance <= radius + 1 {
                map.set_visibility(x, y, TowerTileVisibility::Visible);
            }
        }
    }
}
