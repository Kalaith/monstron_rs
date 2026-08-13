use macroquad::prelude::*;

use super::{movement_buttons, TowerAction};
use crate::assets;
use crate::state::{
    TowerMapObject, TowerMapObjectKind, TowerMapState, TowerRunState, TowerTileKind,
    TowerTileVisibility,
};
use crate::ui;
use macroquad_toolkit::ui::draw_ui_text_ex;

const VIEWPORT_TILES_W: u32 = 18;
const VIEWPORT_TILES_H: u32 = 10;

pub(super) fn draw_map_world(run: &TowerRunState) {
    let map = &run.map;
    if map.is_empty() {
        draw_rectangle(0.0, 0.0, ui::VIEW_WIDTH, ui::VIEW_HEIGHT, rgba(8, 10, 12));
        ui::draw_centered_text(
            "Map data is being rebuilt.",
            ui::VIEW_WIDTH * 0.5,
            ui::VIEW_HEIGHT * 0.5,
            26,
            ui::TEXT_DIM,
        );
        return;
    }

    let map_area = Rect::new(0.0, 0.0, ui::VIEW_WIDTH, ui::VIEW_HEIGHT);
    let minimap_rect = Rect::new(ui::VIEW_WIDTH - 206.0, 72.0, 188.0, 138.0);
    draw_map_viewport(map, map_area);
    draw_minimap(map, minimap_rect);
    draw_legend(488.0, 42.0);
    draw_movement_controls();
}

pub(super) fn world_tap_action(run: &TowerRunState) -> Option<super::TowerAction> {
    if !is_mouse_button_pressed(MouseButton::Left) || run.map.is_empty() {
        return None;
    }
    let (mouse_x, mouse_y) = mouse_position();
    if mouse_x < 212.0 || mouse_x > 1020.0 || mouse_y < 184.0 || mouse_y > 610.0 {
        return None;
    }
    let dx = if mouse_x > 616.0 {
        1
    } else if mouse_x < 500.0 {
        -1
    } else {
        0
    };
    let dy = if mouse_y > 430.0 {
        1
    } else if mouse_y < 315.0 {
        -1
    } else {
        0
    };
    if dx == 0 && dy == 0 {
        None
    } else {
        Some(super::TowerAction::TapMove(dx, dy))
    }
}

fn draw_map_viewport(map: &TowerMapState, area: Rect) {
    let visible_w = map.width.min(VIEWPORT_TILES_W);
    let visible_h = map.height.min(VIEWPORT_TILES_H);
    let start_x = viewport_start(map.player_x, map.width, visible_w);
    let start_y = viewport_start(map.player_y, map.height, visible_h);

    let tile_size = (area.w / visible_w as f32)
        .min(area.h / visible_h as f32)
        .floor()
        .max(18.0);
    let map_w = tile_size * visible_w as f32;
    let map_h = tile_size * visible_h as f32;
    let origin_x = area.x + (area.w - map_w) * 0.5;
    let origin_y = area.y + (area.h - map_h) * 0.5;

    draw_rectangle(0.0, 0.0, ui::VIEW_WIDTH, ui::VIEW_HEIGHT, rgba(7, 9, 10));

    // The authoritative tile grid remains active for movement and fog logic,
    // but the presentation is now room-scale: connected stone paths replace
    // the old debug-grid dominance beneath the illustrated modules.
    draw_room_connectors(map, origin_x, origin_y, start_x, start_y, tile_size);
    draw_room_modules(map, origin_x, origin_y, start_x, start_y, tile_size);

    for object in &map.objects {
        if object.x >= start_x
            && object.x < start_x + visible_w
            && object.y >= start_y
            && object.y < start_y + visible_h
            && map.is_visible(object.x, object.y)
        {
            draw_map_object(
                map,
                object,
                origin_x - start_x as f32 * tile_size,
                origin_y - start_y as f32 * tile_size,
                tile_size,
            );
        }
    }

    draw_player(
        map,
        origin_x - start_x as f32 * tile_size,
        origin_y - start_y as f32 * tile_size,
        tile_size,
    );
}

fn draw_room_connectors(
    map: &TowerMapState,
    origin_x: f32,
    origin_y: f32,
    start_x: u32,
    start_y: u32,
    tile_size: f32,
) {
    for pair in map.rooms.windows(2) {
        let a = pair[0].center();
        let b = pair[1].center();
        if map.visibility_at(a.0, a.1) == TowerTileVisibility::Hidden
            && map.visibility_at(b.0, b.1) == TowerTileVisibility::Hidden
        {
            continue;
        }
        let ax = origin_x + a.0.saturating_sub(start_x) as f32 * tile_size + tile_size * 0.5;
        let ay = origin_y + a.1.saturating_sub(start_y) as f32 * tile_size + tile_size * 0.5;
        let bx = origin_x + b.0.saturating_sub(start_x) as f32 * tile_size + tile_size * 0.5;
        let by = origin_y + b.1.saturating_sub(start_y) as f32 * tile_size + tile_size * 0.5;
        draw_line(ax, ay, bx, ay, tile_size * 0.42, rgba(34, 47, 45));
        draw_line(bx, ay, bx, by, tile_size * 0.42, rgba(34, 47, 45));
        draw_line(ax, ay, bx, ay, tile_size * 0.12, rgba(77, 112, 92));
        draw_line(bx, ay, bx, by, tile_size * 0.12, rgba(77, 112, 92));
    }
}

fn draw_room_modules(
    map: &TowerMapState,
    origin_x: f32,
    origin_y: f32,
    start_x: u32,
    start_y: u32,
    tile_size: f32,
) {
    for room in map.rooms.iter().copied() {
        let room_x = origin_x + (room.start_x.saturating_sub(start_x)) as f32 * tile_size;
        let room_y = origin_y + (room.start_y.saturating_sub(start_y)) as f32 * tile_size;
        let room_w = room.width as f32 * tile_size;
        let room_h = room.height as f32 * tile_size;
        if room_x + room_w < 0.0
            || room_y + room_h < 0.0
            || room_x > ui::VIEW_WIDTH
            || room_y > ui::VIEW_HEIGHT
        {
            continue;
        }
        let visibility = map.visibility_at(room.center().0, room.center().1);
        if visibility == TowerTileVisibility::Hidden {
            assets::draw_dungeon_fog(
                0,
                room_x - tile_size * 0.3,
                room_y - tile_size * 0.3,
                room_w + tile_size * 0.6,
                room_h + tile_size * 0.6,
            );
            continue;
        }
        assets::draw_dungeon_room(
            map.floor,
            room_purpose(map, room),
            room_x,
            room_y,
            room_w,
            room_h,
        );
        draw_room_special_scene(map, room, room_x, room_y, room_w, room_h);
        if visibility == TowerTileVisibility::Explored {
            draw_rectangle(
                room_x,
                room_y,
                room_w,
                room_h,
                Color::from_rgba(15, 23, 28, 108),
            );
        }
    }
}

fn draw_room_special_scene(
    map: &TowerMapState,
    room: crate::state::TowerRoom,
    room_x: f32,
    room_y: f32,
    room_w: f32,
    room_h: f32,
) {
    let object = map.objects.iter().find(|object| {
        object.x >= room.start_x
            && object.x < room.start_x + room.width
            && object.y >= room.start_y
            && object.y < room.start_y + room.height
    });
    let Some(object) = object else { return };
    let inset_x = room_x + room_w * 0.28;
    let inset_y = room_y + room_h * 0.22;
    let inset_w = room_w * 0.44;
    let inset_h = room_h * 0.44;
    match object.kind {
        TowerMapObjectKind::Boss => assets::draw_boss_reward(0, inset_x, inset_y, inset_w, inset_h),
        TowerMapObjectKind::Stairs | TowerMapObjectKind::Exit => {
            assets::draw_recovery_scene(4, inset_x, inset_y, inset_w, inset_h)
        }
        TowerMapObjectKind::Enemy => {
            assets::draw_dungeon_hazard(0, inset_x, inset_y, inset_w, inset_h)
        }
        _ => {}
    }
}

fn room_purpose(map: &TowerMapState, room: crate::state::TowerRoom) -> usize {
    let object = map.objects.iter().find(|object| {
        object.x >= room.start_x
            && object.x < room.start_x + room.width
            && object.y >= room.start_y
            && object.y < room.start_y + room.height
    });
    match object.map(|object| object.kind) {
        Some(TowerMapObjectKind::Egg) => 3,
        Some(TowerMapObjectKind::Loot) => 1,
        Some(TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss) => 2,
        Some(TowerMapObjectKind::Stairs | TowerMapObjectKind::Exit) => 4,
        None => 0,
    }
}

fn viewport_start(center: u32, total: u32, viewport: u32) -> u32 {
    if total <= viewport {
        0
    } else {
        center.saturating_sub(viewport / 2).min(total - viewport)
    }
}

fn rgba(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

fn draw_minimap(map: &TowerMapState, rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(8, 10, 12, 225),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, ui::PANEL_EDGE);

    let tile_size = (rect.w / map.width as f32).min(rect.h / map.height as f32);
    let map_w = tile_size * map.width as f32;
    let map_h = tile_size * map.height as f32;
    let origin_x = rect.x + (rect.w - map_w) * 0.5;
    let origin_y = rect.y + (rect.h - map_h) * 0.5;

    for y in 0..map.height {
        for x in 0..map.width {
            let visibility = map.visibility_at(x, y);
            if visibility == TowerTileVisibility::Hidden {
                continue;
            }
            draw_rectangle(
                origin_x + x as f32 * tile_size,
                origin_y + y as f32 * tile_size,
                tile_size.max(1.0),
                tile_size.max(1.0),
                minimap_tile_color(map.tile_at(x, y), visibility),
            );
        }
    }

    for object in &map.objects {
        if !should_show_on_minimap(map, object) {
            continue;
        }
        let center_x = origin_x + object.x as f32 * tile_size + tile_size * 0.5;
        let center_y = origin_y + object.y as f32 * tile_size + tile_size * 0.5;
        draw_circle(
            center_x,
            center_y,
            tile_size.max(2.0) * 0.72,
            object_color(object.kind),
        );
    }

    let player_x = origin_x + map.player_x as f32 * tile_size + tile_size * 0.5;
    let player_y = origin_y + map.player_y as f32 * tile_size + tile_size * 0.5;
    draw_circle(
        player_x,
        player_y,
        tile_size.max(2.0),
        Color::from_rgba(238, 241, 213, 255),
    );
}

fn minimap_tile_color(tile: TowerTileKind, visibility: TowerTileVisibility) -> Color {
    match visibility {
        TowerTileVisibility::Hidden => Color::from_rgba(7, 9, 11, 255),
        TowerTileVisibility::Explored => match tile {
            TowerTileKind::Wall => Color::from_rgba(18, 22, 25, 255),
            TowerTileKind::Floor => Color::from_rgba(41, 54, 50, 255),
            TowerTileKind::Corridor => Color::from_rgba(34, 46, 43, 255),
        },
        TowerTileVisibility::Visible => match tile {
            TowerTileKind::Wall => Color::from_rgba(28, 34, 38, 255),
            TowerTileKind::Floor => Color::from_rgba(88, 115, 98, 255),
            TowerTileKind::Corridor => Color::from_rgba(67, 92, 82, 255),
        },
    }
}

fn should_show_on_minimap(map: &TowerMapState, object: &TowerMapObject) -> bool {
    match object.kind {
        TowerMapObjectKind::Stairs | TowerMapObjectKind::Exit => {
            map.is_discovered(object.x, object.y)
        }
        TowerMapObjectKind::Loot
        | TowerMapObjectKind::Egg
        | TowerMapObjectKind::Enemy
        | TowerMapObjectKind::Boss => map.is_visible(object.x, object.y),
    }
}

fn object_color(kind: TowerMapObjectKind) -> Color {
    match kind {
        TowerMapObjectKind::Loot => Color::from_rgba(213, 169, 80, 255),
        TowerMapObjectKind::Egg => Color::from_rgba(104, 162, 179, 255),
        TowerMapObjectKind::Enemy => Color::from_rgba(166, 65, 72, 255),
        TowerMapObjectKind::Boss => Color::from_rgba(116, 42, 74, 255),
        TowerMapObjectKind::Stairs => Color::from_rgba(118, 198, 178, 255),
        TowerMapObjectKind::Exit => Color::from_rgba(95, 162, 95, 255),
    }
}

fn draw_map_object(
    map: &TowerMapState,
    object: &TowerMapObject,
    origin_x: f32,
    origin_y: f32,
    tile_size: f32,
) {
    let center_x = origin_x + object.x as f32 * tile_size + tile_size * 0.5;
    let center_y = origin_y + object.y as f32 * tile_size + tile_size * 0.5;
    let pulse = (get_time() as f32 * 2.4 + object.x as f32 * 0.7).sin() * 0.04 + 1.0;
    let shadow = Color::from_rgba(3, 6, 6, 145);
    draw_ellipse(
        center_x,
        center_y + tile_size * 0.3,
        tile_size * 0.38,
        tile_size * 0.14,
        0.0,
        shadow,
    );

    match object.kind {
        TowerMapObjectKind::Loot => {
            draw_object_glow(center_x, center_y, tile_size * 0.42, rgba(218, 163, 69));
            assets::draw_landmark_scene(
                0,
                center_x - tile_size * 0.58,
                center_y - tile_size * 0.58,
                tile_size * 1.16,
                tile_size * 1.16,
            );
            assets::draw_dungeon_feature(
                1,
                center_x - tile_size * 0.54,
                center_y - tile_size * 0.56,
                tile_size * 1.08,
                tile_size * 1.08,
            );
        }
        TowerMapObjectKind::Egg => {
            draw_object_glow(center_x, center_y, tile_size * 0.42, rgba(99, 194, 188));
            assets::draw_egg_badge(
                &object.egg_type_id,
                center_x - tile_size * 0.38,
                center_y - tile_size * 0.42,
                tile_size * 0.76 * pulse,
            );
        }
        TowerMapObjectKind::Enemy => {
            draw_object_glow(center_x, center_y, tile_size * 0.44, rgba(170, 62, 68));
            assets::draw_landmark_scene(
                2,
                center_x - tile_size * 0.58,
                center_y - tile_size * 0.58,
                tile_size * 1.16,
                tile_size * 1.16,
            );
            let index = (map.floor.saturating_sub(1) as usize).min(4);
            assets::draw_dungeon_enemy(
                index,
                center_x - tile_size * 0.55,
                center_y - tile_size * 0.62,
                tile_size * 1.1,
                tile_size * 1.1,
            );
        }
        TowerMapObjectKind::Boss => {
            draw_object_glow(center_x, center_y, tile_size * 0.6, rgba(189, 63, 89));
            assets::draw_landmark_scene(
                5,
                center_x - tile_size * 0.8,
                center_y - tile_size * 0.8,
                tile_size * 1.6,
                tile_size * 1.6,
            );
            assets::draw_dungeon_enemy(
                5,
                center_x - tile_size * 0.72,
                center_y - tile_size * 0.86,
                tile_size * 1.44,
                tile_size * 1.44,
            );
        }
        TowerMapObjectKind::Stairs => {
            draw_object_glow(center_x, center_y, tile_size * 0.52, rgba(85, 189, 177));
            assets::draw_landmark_scene(
                4,
                center_x - tile_size * 0.62,
                center_y - tile_size * 0.62,
                tile_size * 1.24,
                tile_size * 1.24,
            );
            assets::draw_dungeon_feature(
                2,
                center_x - tile_size * 0.65,
                center_y - tile_size * 0.68,
                tile_size * 1.3,
                tile_size * 1.3,
            );
        }
        TowerMapObjectKind::Exit => {
            draw_object_glow(center_x, center_y, tile_size * 0.58, rgba(104, 196, 211));
            assets::draw_dungeon_feature(
                3,
                center_x - tile_size * 0.67,
                center_y - tile_size * 0.75,
                tile_size * 1.34,
                tile_size * 1.34,
            );
        }
    }
}

fn draw_object_glow(x: f32, y: f32, radius: f32, color: Color) {
    let pulse = (get_time() as f32 * 2.0).sin() * 0.08 + 0.18;
    draw_circle(x, y, radius, Color::new(color.r, color.g, color.b, pulse));
}

fn draw_player(map: &TowerMapState, origin_x: f32, origin_y: f32, tile_size: f32) {
    let center_x = origin_x + map.player_x as f32 * tile_size + tile_size * 0.5;
    let center_y = origin_y + map.player_y as f32 * tile_size + tile_size * 0.5;
    let pulse = (get_time() as f32 * 3.0).sin() * 1.5;
    draw_circle(
        center_x,
        center_y,
        tile_size * 0.55 + pulse,
        Color::from_rgba(72, 206, 197, 70),
    );
    draw_circle_lines(
        center_x,
        center_y,
        tile_size * 0.48 + pulse,
        2.0,
        Color::from_rgba(223, 217, 152, 230),
    );
    assets::draw_party_marker(
        0,
        center_x - tile_size * 0.78,
        center_y - tile_size * 0.58,
        tile_size * 1.56,
        tile_size * 1.0,
    );
}

fn draw_legend(x: f32, y: f32) {
    let entries = [
        ("You", Color::from_rgba(238, 241, 213, 255)),
        ("Enemy", Color::from_rgba(166, 65, 72, 255)),
        ("Boss", Color::from_rgba(116, 42, 74, 255)),
        ("Egg", Color::from_rgba(104, 162, 179, 255)),
        ("Cache", Color::from_rgba(213, 169, 80, 255)),
        ("Stairs", Color::from_rgba(118, 198, 178, 255)),
        ("Exit", Color::from_rgba(95, 162, 95, 255)),
    ];

    draw_rectangle(
        x - 10.0,
        y - 28.0,
        650.0,
        38.0,
        Color::from_rgba(8, 12, 13, 218),
    );
    draw_rectangle_lines(x - 10.0, y - 28.0, 650.0, 38.0, 1.0, ui::PANEL_EDGE);
    for (index, (label, color)) in entries.iter().enumerate() {
        let item_x = x + index as f32 * 91.0;
        draw_circle(item_x, y - 6.0, 6.0, *color);
        draw_ui_text_ex(
            label,
            item_x + 12.0,
            y,
            TextParams {
                font_size: 15,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    }
}

fn draw_movement_controls() {
    for (action, rect) in movement_buttons() {
        let label = match action {
            TowerAction::Move(0, -1) => "N",
            TowerAction::Move(0, 1) => "S",
            TowerAction::Move(-1, 0) => "W",
            TowerAction::Move(1, 0) => "E",
            _ => "",
        };
        ui::draw_button(rect, label, true);
    }
}
