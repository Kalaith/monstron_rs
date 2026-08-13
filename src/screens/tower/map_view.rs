use macroquad::prelude::*;

use super::TowerAction;
use crate::assets::{self, DungeonBiome, DungeonRoomPurpose};
use crate::state::{
    TowerMapObject, TowerMapObjectKind, TowerMapState, TowerRoom, TowerRunState,
    TowerTileVisibility,
};
use crate::ui;

const WORLD: Rect = Rect::new(138.0, 52.0, 1006.0, 602.0);

#[derive(Clone, Copy)]
struct WorldTransform {
    origin: Vec2,
    scale: Vec2,
}

pub(super) fn draw_map_world(run: &TowerRunState) {
    let map = &run.map;
    draw_world_backdrop(map.floor);
    if map.is_empty() {
        ui::draw_centered_text(
            "The tower is rebuilding this floor...",
            ui::VIEW_WIDTH * 0.5,
            ui::VIEW_HEIGHT * 0.5,
            24,
            ui::TEXT_DIM,
        );
        return;
    }

    let transform = world_transform(map);
    draw_connectors(map, transform);
    draw_rooms(map, transform);
    draw_objects(map, transform);
    draw_party(map, transform);
    draw_world_fog(map);
}

pub(super) fn world_tap_action(run: &TowerRunState) -> Option<TowerAction> {
    if !is_mouse_button_pressed(MouseButton::Left) || run.map.is_empty() {
        return None;
    }
    let mouse = macroquad_toolkit::ui::virtual_mouse_position(ui::VIEW_WIDTH, ui::VIEW_HEIGHT);
    if !WORLD.contains(mouse) {
        return None;
    }

    let map = &run.map;
    let transform = world_transform(map);
    let selected = map
        .rooms
        .iter()
        .copied()
        .filter(|room| room_rect(*room, transform).contains(mouse))
        .min_by_key(|room| {
            let center = room.center();
            map.player_x.abs_diff(center.0) + map.player_y.abs_diff(center.1)
        })?;
    let target = selected.center();
    let dx = axis_step(target.0 as i32 - map.player_x as i32);
    let dy = axis_step(target.1 as i32 - map.player_y as i32);
    if dx == 0 && dy == 0 {
        None
    } else if (target.0 as i32 - map.player_x as i32).abs()
        >= (target.1 as i32 - map.player_y as i32).abs()
    {
        Some(TowerAction::TapMove(dx, 0))
    } else {
        Some(TowerAction::TapMove(0, dy))
    }
}

fn axis_step(value: i32) -> i32 {
    value.signum()
}

fn draw_world_backdrop(floor: u32) {
    draw_rectangle(
        0.0,
        0.0,
        ui::VIEW_WIDTH,
        ui::VIEW_HEIGHT,
        color(4, 9, 11, 255),
    );
    for band in 0..9 {
        let inset = band as f32 * 22.0;
        let alpha = 18_u8.saturating_sub(band as u8);
        draw_ellipse(
            640.0,
            365.0,
            610.0 - inset,
            350.0 - inset * 0.45,
            0.0,
            color(21, 47, 47, alpha),
        );
    }
    draw_background_haze(floor);
}

fn draw_background_haze(floor: u32) {
    let drift = floor as f32 * 1.7;
    for cloud in 0..18 {
        let phase = cloud as f32 * 2.13 + drift;
        let x = 80.0 + ((phase * 0.71).sin() + 1.0) * 560.0;
        let y = 80.0 + ((phase * 0.43).cos() + 1.0) * 280.0;
        draw_ellipse(x, y, 150.0, 58.0, phase * 0.04, color(43, 66, 65, 10));
    }
}

fn world_transform(map: &TowerMapState) -> WorldTransform {
    let scale_x = WORLD.w / map.width.max(1) as f32;
    let scale_y = WORLD.h / map.height.max(1) as f32;
    WorldTransform {
        origin: vec2(WORLD.x, WORLD.y),
        scale: vec2(scale_x, scale_y),
    }
}

fn room_rect(room: TowerRoom, transform: WorldTransform) -> Rect {
    let center = room.center();
    let width = (room.width as f32 * transform.scale.x * 1.32).clamp(210.0, 300.0);
    let height = (room.height as f32 * transform.scale.y * 1.34).clamp(172.0, 234.0);
    Rect::new(
        transform.origin.x + center.0 as f32 * transform.scale.x - width * 0.5,
        transform.origin.y + center.1 as f32 * transform.scale.y - height * 0.5,
        width,
        height,
    )
}

fn room_center(room: TowerRoom, transform: WorldTransform) -> Vec2 {
    let rect = room_rect(room, transform);
    vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.58)
}

fn draw_connectors(map: &TowerMapState, transform: WorldTransform) {
    for (index, pair) in map.rooms.windows(2).enumerate() {
        let a = room_center(pair[0], transform);
        let b = room_center(pair[1], transform);
        let visibility = pair_visibility(map, pair[0], pair[1]);
        if visibility == TowerTileVisibility::Hidden && index > 5 {
            continue;
        }
        let alpha = if visibility == TowerTileVisibility::Visible {
            235
        } else {
            105
        };
        draw_line(
            a.x,
            a.y + 7.0,
            b.x,
            a.y + 7.0,
            34.0,
            color(10, 15, 14, alpha),
        );
        draw_line(b.x, a.y + 7.0, b.x, b.y, 34.0, color(10, 15, 14, alpha));
        draw_line(a.x, a.y, b.x, a.y, 22.0, color(47, 55, 41, alpha));
        draw_line(b.x, a.y, b.x, b.y, 22.0, color(47, 55, 41, alpha));
        draw_line(a.x, a.y, b.x, a.y, 3.0, color(111, 96, 55, alpha / 2));
        draw_line(b.x, a.y, b.x, b.y, 3.0, color(111, 96, 55, alpha / 2));
    }
}

fn pair_visibility(map: &TowerMapState, a: TowerRoom, b: TowerRoom) -> TowerTileVisibility {
    let va = map.visibility_at(a.center().0, a.center().1);
    let vb = map.visibility_at(b.center().0, b.center().1);
    if va == TowerTileVisibility::Visible || vb == TowerTileVisibility::Visible {
        TowerTileVisibility::Visible
    } else if va == TowerTileVisibility::Explored || vb == TowerTileVisibility::Explored {
        TowerTileVisibility::Explored
    } else {
        TowerTileVisibility::Hidden
    }
}

fn draw_rooms(map: &TowerMapState, transform: WorldTransform) {
    let biome = DungeonBiome::for_floor(map.floor);
    for (index, room) in map.rooms.iter().copied().enumerate() {
        let rect = room_rect(room, transform);
        if rect.x + rect.w < WORLD.x || rect.x > WORLD.x + WORLD.w {
            continue;
        }
        let visibility = map.visibility_at(room.center().0, room.center().1);
        let purpose = room_purpose(map, room, index);
        let tint = match visibility {
            TowerTileVisibility::Visible => WHITE,
            TowerTileVisibility::Explored => color(120, 133, 119, 220),
            TowerTileVisibility::Hidden => color(132, 143, 118, 225),
        };
        draw_room_shadow(rect, visibility);
        assets::draw_dungeon_room(biome, purpose, rect.x, rect.y, rect.w, rect.h, tint);
        if visibility == TowerTileVisibility::Hidden {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color(3, 12, 14, 30));
            draw_room_mist(rect, index);
        } else if visibility == TowerTileVisibility::Visible {
            draw_light_pool(
                rect.x + rect.w * 0.5,
                rect.y + rect.h * 0.56,
                rect.w * 0.38,
                purpose,
            );
        }
    }
}

fn draw_room_mist(rect: Rect, seed: usize) {
    for cloud in 0..4 {
        let phase = (seed * 7 + cloud * 11) as f32;
        let x = rect.x + rect.w * (0.12 + ((phase * 0.37).sin() + 1.0) * 0.38);
        let y = rect.y + rect.h * (0.16 + ((phase * 0.23).cos() + 1.0) * 0.32);
        draw_ellipse(
            x,
            y,
            rect.w * (0.24 + cloud as f32 * 0.025),
            rect.h * 0.16,
            0.0,
            color(31, 49, 51, 15),
        );
    }
}

fn draw_room_shadow(rect: Rect, visibility: TowerTileVisibility) {
    let alpha = if visibility == TowerTileVisibility::Hidden {
        105
    } else {
        205
    };
    draw_ellipse(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.74,
        rect.w * 0.55,
        rect.h * 0.35,
        0.0,
        color(0, 0, 0, alpha),
    );
}

fn draw_light_pool(x: f32, y: f32, radius: f32, purpose: DungeonRoomPurpose) {
    let glow = match purpose {
        DungeonRoomPurpose::Nest => (174, 124, 205),
        DungeonRoomPurpose::Encounter => (207, 93, 60),
        DungeonRoomPurpose::Shrine | DungeonRoomPurpose::Traversal => (80, 190, 208),
        _ => (232, 173, 82),
    };
    for ring in (1..=5).rev() {
        let fraction = ring as f32 / 5.0;
        draw_circle(x, y, radius * fraction, color(glow.0, glow.1, glow.2, 9));
    }
}

fn room_purpose(map: &TowerMapState, room: TowerRoom, index: usize) -> DungeonRoomPurpose {
    match object_in_room(map, room).map(|object| object.kind) {
        Some(TowerMapObjectKind::Egg) => DungeonRoomPurpose::Nest,
        Some(TowerMapObjectKind::Loot) => DungeonRoomPurpose::Cache,
        Some(TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss) => DungeonRoomPurpose::Encounter,
        Some(TowerMapObjectKind::Stairs | TowerMapObjectKind::Exit) => {
            DungeonRoomPurpose::Traversal
        }
        None if index == 0 => DungeonRoomPurpose::Camp,
        None if index % 4 == 0 => DungeonRoomPurpose::Shrine,
        None => DungeonRoomPurpose::Traversal,
    }
}

fn object_in_room(map: &TowerMapState, room: TowerRoom) -> Option<&TowerMapObject> {
    map.objects.iter().find(|object| {
        object.x >= room.start_x
            && object.x < room.start_x + room.width
            && object.y >= room.start_y
            && object.y < room.start_y + room.height
    })
}

fn draw_objects(map: &TowerMapState, transform: WorldTransform) {
    for object in &map.objects {
        if !map.is_visible(object.x, object.y) {
            continue;
        }
        let x = transform.origin.x + object.x as f32 * transform.scale.x;
        let y = transform.origin.y + object.y as f32 * transform.scale.y;
        let size = match object.kind {
            TowerMapObjectKind::Boss => 82.0,
            TowerMapObjectKind::Stairs | TowerMapObjectKind::Exit => 70.0,
            _ => 58.0,
        };
        draw_object_glow(x, y, size, object.kind);
        match object.kind {
            TowerMapObjectKind::Loot => {
                assets::draw_dungeon_feature(1, x - size * 0.5, y - size * 0.55, size, size)
            }
            TowerMapObjectKind::Egg => assets::draw_egg_badge(
                &object.egg_type_id,
                x - size * 0.42,
                y - size * 0.5,
                size * 0.84,
            ),
            TowerMapObjectKind::Enemy => assets::draw_dungeon_enemy(
                (map.floor - 1) as usize,
                x - size * 0.5,
                y - size * 0.6,
                size,
                size,
            ),
            TowerMapObjectKind::Boss => {
                assets::draw_dungeon_enemy(5, x - size * 0.5, y - size * 0.64, size, size)
            }
            TowerMapObjectKind::Stairs => {
                assets::draw_dungeon_feature(2, x - size * 0.5, y - size * 0.58, size, size)
            }
            TowerMapObjectKind::Exit => {
                assets::draw_dungeon_feature(3, x - size * 0.5, y - size * 0.58, size, size)
            }
        }
    }
}

fn draw_object_glow(x: f32, y: f32, size: f32, kind: TowerMapObjectKind) {
    let rgb = match kind {
        TowerMapObjectKind::Loot => (230, 171, 63),
        TowerMapObjectKind::Egg => (181, 120, 214),
        TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss => (211, 73, 67),
        TowerMapObjectKind::Stairs | TowerMapObjectKind::Exit => (88, 196, 206),
    };
    draw_circle(x, y, size * 0.62, color(rgb.0, rgb.1, rgb.2, 31));
}

fn draw_party(map: &TowerMapState, transform: WorldTransform) {
    let x = transform.origin.x + map.player_x as f32 * transform.scale.x;
    let y = transform.origin.y + map.player_y as f32 * transform.scale.y;
    draw_circle(x, y + 6.0, 52.0, color(245, 194, 83, 28));
    draw_circle_lines(x, y + 6.0, 45.0, 2.0, color(226, 188, 92, 210));
    assets::draw_party_marker(0, x - 68.0, y - 47.0, 136.0, 91.0);
}

fn draw_world_fog(map: &TowerMapState) {
    let explored = map
        .visibility
        .iter()
        .filter(|v| **v != TowerTileVisibility::Hidden)
        .count();
    let ratio = explored as f32 / map.visibility.len().max(1) as f32;
    let edge_alpha = (175.0 - ratio * 80.0) as u8;
    for ring in 0..8 {
        let inset = ring as f32 * 18.0;
        draw_rectangle(
            WORLD.x,
            WORLD.y + inset,
            70.0 - inset * 0.25,
            WORLD.h - inset * 2.0,
            color(3, 9, 12, edge_alpha / 8),
        );
        draw_rectangle(
            WORLD.x + WORLD.w - 70.0 + inset * 0.25,
            WORLD.y + inset,
            70.0,
            WORLD.h - inset * 2.0,
            color(3, 9, 12, edge_alpha / 8),
        );
    }
}

fn color(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}
