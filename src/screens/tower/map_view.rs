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

    for view_y in 0..visible_h {
        for view_x in 0..visible_w {
            let x = start_x + view_x;
            let y = start_y + view_y;
            let visibility = map.visibility_at(x, y);
            let tile = Rect::new(
                origin_x + view_x as f32 * tile_size,
                origin_y + view_y as f32 * tile_size,
                tile_size,
                tile_size,
            );
            draw_dungeon_tile(map, x, y, tile, visibility);
        }
    }

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

fn viewport_start(center: u32, total: u32, viewport: u32) -> u32 {
    if total <= viewport {
        0
    } else {
        center.saturating_sub(viewport / 2).min(total - viewport)
    }
}

fn draw_dungeon_tile(
    map: &TowerMapState,
    x: u32,
    y: u32,
    tile: Rect,
    visibility: TowerTileVisibility,
) {
    let kind = map.tile_at(x, y);
    let variation = tile_variation(map.seed, x, y);
    if visibility == TowerTileVisibility::Hidden {
        let shade = 5 + variation % 4;
        draw_rectangle(
            tile.x,
            tile.y,
            tile.w,
            tile.h,
            rgba(shade, shade + 2, shade + 3),
        );
        return;
    }

    let dim = visibility == TowerTileVisibility::Explored;
    match kind {
        TowerTileKind::Wall => draw_wall_tile(map, x, y, tile, variation, dim),
        TowerTileKind::Floor => draw_floor_tile(map.floor, tile, variation, dim, false),
        TowerTileKind::Corridor => draw_floor_tile(map.floor, tile, variation, dim, true),
    }

    if dim {
        draw_rectangle(
            tile.x,
            tile.y,
            tile.w,
            tile.h,
            Color::from_rgba(5, 8, 10, 126),
        );
    }
}

fn draw_floor_tile(floor: u32, tile: Rect, variation: u8, dim: bool, corridor: bool) {
    let (base, mortar, moss, glow) = floor_palette(floor);
    let color = shift_color(base, (variation % 7) as i16 - 3);
    draw_rectangle(tile.x, tile.y, tile.w, tile.h, color);

    let inset = if corridor { 2.8 } else { 1.2 };
    draw_rectangle_lines(
        tile.x + inset,
        tile.y + inset,
        tile.w - inset * 2.0,
        tile.h - inset * 2.0,
        if corridor { 1.5 } else { 0.8 },
        mortar,
    );
    let seam = 0.28 + (variation % 4) as f32 * 0.13;
    draw_line(
        tile.x + tile.w * seam,
        tile.y + tile.h * 0.08,
        tile.x + tile.w * (seam + 0.1),
        tile.y + tile.h * 0.46,
        0.8,
        mortar,
    );
    if variation % 3 == 0 {
        draw_circle(
            tile.x + tile.w * 0.78,
            tile.y + tile.h * 0.2,
            tile.w * 0.055,
            moss,
        );
    }
    if !dim && variation % 11 == 0 {
        draw_circle(
            tile.x + tile.w * 0.18,
            tile.y + tile.h * 0.72,
            tile.w * 0.035,
            glow,
        );
    }
}

fn draw_wall_tile(map: &TowerMapState, x: u32, y: u32, tile: Rect, variation: u8, dim: bool) {
    let (_, mortar, moss, glow) = floor_palette(map.floor);
    let base = shift_color(
        Color::from_rgba(29, 34, 34, 255),
        (variation % 9) as i16 - 4,
    );
    draw_rectangle(tile.x, tile.y, tile.w, tile.h, base);
    draw_rectangle(
        tile.x + 2.0,
        tile.y + 2.0,
        tile.w - 4.0,
        tile.h * 0.34,
        shift_color(base, 10),
    );
    draw_line(
        tile.x,
        tile.y + tile.h * 0.55,
        tile.x + tile.w,
        tile.y + tile.h * 0.55,
        1.0,
        mortar,
    );
    draw_line(
        tile.x + tile.w * 0.55,
        tile.y + tile.h * 0.55,
        tile.x + tile.w * 0.55,
        tile.y + tile.h,
        0.8,
        mortar,
    );

    if borders_passage(map, x, y) {
        draw_rectangle(tile.x, tile.y + tile.h - 4.0, tile.w, 4.0, moss);
        if !dim && variation % 4 == 0 {
            draw_circle(tile.x + tile.w * 0.5, tile.y + tile.h * 0.76, 2.0, glow);
        }
    }
}

fn borders_passage(map: &TowerMapState, x: u32, y: u32) -> bool {
    [(0_i32, -1_i32), (0, 1), (-1, 0), (1, 0)]
        .iter()
        .any(|(dx, dy)| {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            nx >= 0 && ny >= 0 && map.tile_at(nx as u32, ny as u32).is_passable()
        })
}

fn tile_variation(seed: u64, x: u32, y: u32) -> u8 {
    let mixed =
        seed ^ u64::from(x).wrapping_mul(0x9E37_79B9) ^ u64::from(y).wrapping_mul(0x85EB_CA6B);
    ((mixed ^ (mixed >> 17) ^ (mixed >> 41)) & 0xff) as u8
}

fn floor_palette(floor: u32) -> (Color, Color, Color, Color) {
    match floor {
        4 | 7 => (
            rgba(65, 49, 39),
            rgba(35, 27, 25),
            rgba(75, 69, 42),
            rgba(234, 131, 55),
        ),
        5 | 8 => (
            rgba(43, 64, 65),
            rgba(23, 38, 42),
            rgba(45, 91, 72),
            rgba(73, 211, 207),
        ),
        6 | 9 => (
            rgba(49, 52, 66),
            rgba(27, 29, 40),
            rgba(65, 75, 65),
            rgba(126, 173, 224),
        ),
        10 => (
            rgba(55, 54, 46),
            rgba(29, 31, 27),
            rgba(74, 105, 55),
            rgba(168, 111, 218),
        ),
        _ => (
            rgba(54, 62, 54),
            rgba(29, 35, 32),
            rgba(62, 91, 55),
            rgba(65, 180, 174),
        ),
    }
}

fn shift_color(color: Color, shift: i16) -> Color {
    Color::from_rgba(
        ((color.r * 255.0) as i16)
            .saturating_add(shift)
            .clamp(0, 255) as u8,
        ((color.g * 255.0) as i16)
            .saturating_add(shift)
            .clamp(0, 255) as u8,
        ((color.b * 255.0) as i16)
            .saturating_add(shift)
            .clamp(0, 255) as u8,
        255,
    )
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
