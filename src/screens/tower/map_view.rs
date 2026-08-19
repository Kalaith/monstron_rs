use macroquad::prelude::*;

use super::TowerAction;
use crate::assets::{self, DungeonBiome, DungeonRoomPurpose};
use crate::data::GameData;
use crate::engine::tower_engine;
use crate::state::{
    GameState, TowerMapObject, TowerMapObjectKind, TowerMapState, TowerRoom, TowerRoomKind,
    TowerRunState, TowerTileVisibility,
};
use crate::ui;

const WORLD: Rect = Rect::new(138.0, 52.0, 1006.0, 602.0);

#[derive(Clone, Copy)]
struct WorldTransform {
    origin: Vec2,
    scale: Vec2,
}

pub(super) fn draw_map_world(state: &GameState, data: &GameData, run: &TowerRunState) {
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
    if map.floor == 1 {
        assets::draw_moss_gate_world(WORLD.x, WORLD.y, WORLD.w, WORLD.h);
    } else {
        draw_authoritative_ruin(map, transform);
        draw_rooms(map, transform);
    }
    draw_route_target(map, transform, run.route_target);
    draw_objects(data, map, transform, run.boss_defeated);
    draw_party(state, map, transform);
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
    tower_engine::room_tap_direction(run, target)?;
    Some(TowerAction::RouteTo(target.0, target.1))
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

fn draw_route_target(map: &TowerMapState, transform: WorldTransform, target: Option<(u32, u32)>) {
    let Some(target) = target else {
        return;
    };
    let Some(room) = map.rooms.iter().find(|room| {
        target.0 >= room.start_x
            && target.0 < room.start_x + room.width
            && target.1 >= room.start_y
            && target.1 < room.start_y + room.height
    }) else {
        return;
    };
    let rect = room_rect(*room, transform);
    draw_ellipse_lines(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.6,
        rect.w * 0.38,
        rect.h * 0.28,
        0.0,
        4.0,
        color(232, 173, 82, 220),
    );
}

fn draw_authoritative_ruin(map: &TowerMapState, transform: WorldTransform) {
    draw_water_channels(map);
    for (index, pair) in map.rooms.windows(2).enumerate() {
        let a = room_center(pair[0], transform);
        let b = room_center(pair[1], transform);
        let corner = if (map.seed as usize + index).is_multiple_of(2) {
            vec2(b.x, a.y)
        } else {
            vec2(a.x, b.y)
        };
        draw_cobbled_segment(map, a, corner, index * 2);
        draw_cobbled_segment(map, corner, b, index * 2 + 1);
        draw_courtyard(corner, 46.0, map.seed as usize + index);
    }
    for room in &map.rooms {
        draw_courtyard(room_center(*room, transform), 70.0, room.start_x as usize);
    }
}

fn room_center(room: TowerRoom, transform: WorldTransform) -> Vec2 {
    let rect = room_rect(room, transform);
    vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.62)
}

fn draw_water_channels(map: &TowerMapState) {
    draw_rectangle(WORLD.x, WORLD.y, WORLD.w, WORLD.h, color(4, 23, 29, 168));
    for pool in 0..18 {
        let seed = pool as u32 * 37 + map.floor * 19 + map.seed as u32;
        let x = WORLD.x + (seed % 997) as f32 / 997.0 * WORLD.w;
        let y = WORLD.y + (seed.wrapping_mul(17) % 991) as f32 / 991.0 * WORLD.h;
        draw_circle(x, y, 3.0 + (seed % 5) as f32, color(58, 91, 67, 75));
    }
}

fn draw_cobbled_segment(map: &TowerMapState, start: Vec2, end: Vec2, seed: usize) {
    let delta = end - start;
    let length = delta.length();
    if length < 3.0 {
        return;
    }
    let direction = delta / length;
    let normal = vec2(-direction.y, direction.x);
    draw_line(
        start.x,
        start.y + 6.0,
        end.x,
        end.y + 6.0,
        112.0,
        color(2, 7, 8, 205),
    );
    draw_line(
        start.x,
        start.y,
        end.x,
        end.y,
        100.0,
        color(27, 37, 32, 255),
    );
    draw_line(start.x, start.y, end.x, end.y, 88.0, color(57, 64, 51, 255));
    draw_line(start.x, start.y, end.x, end.y, 3.0, color(111, 108, 75, 80));
    let steps = (length / 24.0).ceil() as usize;
    for step in 0..=steps {
        let t = step as f32 / steps.max(1) as f32;
        let hash = seed * 97 + step * 41 + map.seed as usize;
        for lane in -2_i32..=2 {
            let jitter = ((hash + lane.unsigned_abs() as usize * 13) % 11) as f32 - 5.0;
            let center = start + delta * t + normal * (lane as f32 * 16.0 + jitter);
            let stone_w = 12.0 + (hash % 8) as f32;
            let stone_h = 8.0 + ((hash / 3) % 7) as f32;
            draw_rectangle(
                center.x - stone_w * 0.5,
                center.y - stone_h * 0.5,
                stone_w,
                stone_h,
                color(76, 79, 61, 190),
            );
            draw_rectangle_lines(
                center.x - stone_w * 0.5,
                center.y - stone_h * 0.5,
                stone_w,
                stone_h,
                1.0,
                color(24, 31, 27, 180),
            );
        }
    }
}

fn draw_courtyard(center: Vec2, radius: f32, seed: usize) {
    draw_circle(center.x, center.y + 6.0, radius + 10.0, color(2, 7, 8, 205));
    draw_circle(center.x, center.y, radius, color(55, 62, 49, 255));
    for ring in 1..=3 {
        draw_circle_lines(
            center.x,
            center.y,
            radius * ring as f32 / 3.0,
            2.0,
            color(90, 87, 61, 100),
        );
    }
    for spoke in 0..8 {
        let angle = spoke as f32 * std::f32::consts::TAU / 8.0 + seed as f32 * 0.03;
        draw_line(
            center.x,
            center.y,
            center.x + angle.cos() * radius,
            center.y + angle.sin() * radius,
            1.0,
            color(23, 31, 26, 150),
        );
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
    let stored = match map.room_kind(index) {
        TowerRoomKind::Camp => Some(DungeonRoomPurpose::Camp),
        TowerRoomKind::Nest => Some(DungeonRoomPurpose::Nest),
        TowerRoomKind::Cache => Some(DungeonRoomPurpose::Cache),
        TowerRoomKind::Encounter => Some(DungeonRoomPurpose::Encounter),
        TowerRoomKind::Hazard | TowerRoomKind::Traversal => Some(DungeonRoomPurpose::Traversal),
        TowerRoomKind::Landmark => Some(DungeonRoomPurpose::Shrine),
        TowerRoomKind::Unknown => None,
    };
    if let Some(purpose) = stored {
        return purpose;
    }
    match object_in_room(map, room).map(|object| object.kind) {
        Some(TowerMapObjectKind::Egg) => DungeonRoomPurpose::Nest,
        Some(TowerMapObjectKind::Loot) => DungeonRoomPurpose::Cache,
        Some(TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss) => DungeonRoomPurpose::Encounter,
        Some(TowerMapObjectKind::Hazard) => DungeonRoomPurpose::Traversal,
        Some(TowerMapObjectKind::SpecialLocation) => DungeonRoomPurpose::Shrine,
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

fn draw_objects(
    data: &GameData,
    map: &TowerMapState,
    transform: WorldTransform,
    boss_defeated: bool,
) {
    for object in &map.objects {
        if !map.is_visible(object.x, object.y) {
            continue;
        }
        let position = map_point(map, transform, object.x, object.y);
        let x = position.x;
        let y = position.y;
        let size = match object.kind {
            TowerMapObjectKind::Boss => 82.0,
            TowerMapObjectKind::SpecialLocation => 76.0,
            TowerMapObjectKind::Hazard => 72.0,
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
            TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss => {
                if let Some(enemy) = data.enemy(&object.enemy_id) {
                    if object.wandering {
                        assets::draw_wandering_enemy_visual(
                            enemy.visual,
                            x - size * 0.5,
                            y - size * 0.62,
                            size,
                            size,
                        );
                    } else {
                        assets::draw_dungeon_enemy_visual(
                            enemy.visual,
                            x - size * 0.5,
                            y - size * 0.62,
                            size,
                            size,
                        );
                    }
                }
            }
            TowerMapObjectKind::SpecialLocation => {
                if let Some(location) = data.tower_special_location(&object.special_location_id) {
                    assets::draw_special_location(
                        location.visual,
                        x - size * 0.5,
                        y - size * 0.62,
                        size,
                        size,
                    );
                } else {
                    assets::draw_dungeon_feature(4, x - size * 0.5, y - size * 0.6, size, size);
                }
            }
            TowerMapObjectKind::Hazard => {
                if let Some(hazard) = data.tower_hazard(&object.hazard_id) {
                    assets::draw_tower_hazard(
                        hazard.visual,
                        x - size * 0.5,
                        y - size * 0.58,
                        size,
                        size,
                    );
                }
            }
            TowerMapObjectKind::Stairs => assets::draw_escalation_landmark(
                DungeonBiome::for_floor(map.floor),
                x - size * 0.5,
                y - size * 0.58,
                size,
                size,
            ),
            TowerMapObjectKind::Exit => {
                let boss_floor = data
                    .tower_floor(map.floor)
                    .is_some_and(|floor| floor.is_boss_floor);
                if boss_floor && !boss_defeated {
                    assets::draw_escalation_landmark(
                        DungeonBiome::for_floor(map.floor),
                        x - size * 0.5,
                        y - size * 0.58,
                        size,
                        size,
                    )
                } else {
                    assets::draw_escape_cue(
                        DungeonBiome::for_floor(map.floor),
                        x - size * 0.5,
                        y - size * 0.58,
                        size,
                        size,
                    )
                }
            }
        }
    }
}

fn draw_object_glow(x: f32, y: f32, size: f32, kind: TowerMapObjectKind) {
    let rgb = match kind {
        TowerMapObjectKind::Loot => (230, 171, 63),
        TowerMapObjectKind::Egg => (181, 120, 214),
        TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss => (211, 73, 67),
        TowerMapObjectKind::SpecialLocation => (99, 211, 168),
        TowerMapObjectKind::Hazard => (232, 135, 57),
        TowerMapObjectKind::Stairs | TowerMapObjectKind::Exit => (88, 196, 206),
    };
    draw_circle(x, y, size * 0.62, color(rgb.0, rgb.1, rgb.2, 31));
}

fn draw_party(state: &GameState, map: &TowerMapState, transform: WorldTransform) {
    let position = map_point(map, transform, map.player_x, map.player_y);
    let x = position.x;
    let y = position.y;
    draw_circle(x, y + 4.0, 40.0, color(245, 194, 83, 25));
    let members: Vec<_> = state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .filter_map(|id| state.monster_roster.monster(*id))
        .take(3)
        .collect();
    for (index, monster) in members.iter().enumerate() {
        let offset = (index as f32 - (members.len().saturating_sub(1)) as f32 * 0.5) * 32.0;
        assets::draw_monster_sprite(&monster.species_id, x + offset - 25.0, y - 30.0, 50.0);
    }
}

fn map_point(map: &TowerMapState, transform: WorldTransform, x: u32, y: u32) -> Vec2 {
    if map.floor == 1 {
        vec2(
            WORLD.x + (x as f32 + 0.5) * WORLD.w / map.width.max(1) as f32,
            WORLD.y + (y as f32 + 0.5) * WORLD.h / map.height.max(1) as f32,
        )
    } else {
        vec2(
            transform.origin.x + x as f32 * transform.scale.x,
            transform.origin.y + y as f32 * transform.scale.y,
        )
    }
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
