use macroquad::prelude::*;

use super::{movement_buttons, TowerAction};
use crate::assets;
use crate::state::{
    TowerMapObject, TowerMapObjectKind, TowerMapState, TowerRunState, TowerTileKind,
    TowerTileVisibility,
};
use crate::ui;
use macroquad_toolkit::ui::draw_ui_text_ex;

const VIEWPORT_TILES_W: u32 = 17;
const VIEWPORT_TILES_H: u32 = 13;

pub(super) fn draw_map_panel(run: &TowerRunState) {
    let rect = Rect::new(32.0, 124.0, 776.0, 494.0);
    ui::draw_panel(rect);

    let map = &run.map;
    if map.is_empty() {
        ui::draw_centered_text(
            "Map data is being rebuilt.",
            rect.x + rect.w * 0.5,
            rect.y + rect.h * 0.5,
            26,
            ui::TEXT_DIM,
        );
        return;
    }

    let map_area = Rect::new(rect.x + 36.0, rect.y + 26.0, 548.0, 386.0);
    let minimap_rect = Rect::new(rect.x + 604.0, rect.y + 34.0, 146.0, 116.0);
    draw_map_viewport(map, map_area);
    draw_minimap(map, minimap_rect);
    draw_legend(rect.x + 24.0, rect.y + 444.0);
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

    draw_rectangle(
        origin_x - 6.0,
        origin_y - 6.0,
        map_w + 12.0,
        map_h + 12.0,
        Color::from_rgba(8, 10, 12, 225),
    );

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
            draw_rectangle(
                tile.x,
                tile.y,
                tile.w,
                tile.h,
                tile_color(map.tile_at(x, y), visibility),
            );
            if visibility == TowerTileVisibility::Visible {
                draw_rectangle_lines(
                    tile.x,
                    tile.y,
                    tile.w,
                    tile.h,
                    0.6,
                    Color::from_rgba(94, 114, 107, 90),
                );
            }
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
    draw_rectangle_lines(
        origin_x - 6.0,
        origin_y - 6.0,
        map_w + 12.0,
        map_h + 12.0,
        2.0,
        Color::from_rgba(92, 112, 104, 255),
    );
}

fn viewport_start(center: u32, total: u32, viewport: u32) -> u32 {
    if total <= viewport {
        0
    } else {
        center.saturating_sub(viewport / 2).min(total - viewport)
    }
}

fn tile_color(tile: TowerTileKind, visibility: TowerTileVisibility) -> Color {
    match visibility {
        TowerTileVisibility::Hidden => Color::from_rgba(7, 9, 11, 255),
        TowerTileVisibility::Explored => match tile {
            TowerTileKind::Wall => Color::from_rgba(14, 17, 20, 255),
            TowerTileKind::Floor => Color::from_rgba(34, 43, 42, 255),
            TowerTileKind::Corridor => Color::from_rgba(28, 36, 35, 255),
        },
        TowerTileVisibility::Visible => match tile {
            TowerTileKind::Wall => Color::from_rgba(22, 27, 32, 255),
            TowerTileKind::Floor => Color::from_rgba(72, 85, 80, 255),
            TowerTileKind::Corridor => Color::from_rgba(52, 65, 61, 255),
        },
    }
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

fn draw_map_object(object: &TowerMapObject, origin_x: f32, origin_y: f32, tile_size: f32) {
    let center_x = origin_x + object.x as f32 * tile_size + tile_size * 0.5;
    let center_y = origin_y + object.y as f32 * tile_size + tile_size * 0.5;
    let radius = tile_size * 0.34;

    match object.kind {
        TowerMapObjectKind::Loot => {
            draw_rectangle(
                center_x - radius * 0.7,
                center_y - radius * 0.7,
                radius * 1.4,
                radius * 1.4,
                Color::from_rgba(213, 169, 80, 255),
            );
            draw_rectangle_lines(
                center_x - radius * 0.7,
                center_y - radius * 0.7,
                radius * 1.4,
                radius * 1.4,
                1.0,
                Color::from_rgba(85, 50, 24, 255),
            );
        }
        TowerMapObjectKind::Egg => {
            assets::draw_egg_badge(
                object.palette_seed,
                center_x - radius,
                center_y - radius * 1.1,
                radius * 2.0,
            );
        }
        TowerMapObjectKind::Enemy => {
            draw_circle(
                center_x,
                center_y,
                radius,
                Color::from_rgba(166, 65, 72, 255),
            );
            draw_circle(
                center_x - radius * 0.3,
                center_y - radius * 0.1,
                radius * 0.13,
                BLACK,
            );
            draw_circle(
                center_x + radius * 0.3,
                center_y - radius * 0.1,
                radius * 0.13,
                BLACK,
            );
        }
        TowerMapObjectKind::Boss => {
            draw_circle(
                center_x,
                center_y,
                radius * 1.25,
                Color::from_rgba(116, 42, 74, 255),
            );
            draw_circle_lines(
                center_x,
                center_y,
                radius * 1.25,
                2.0,
                Color::from_rgba(236, 129, 91, 255),
            );
        }
        TowerMapObjectKind::Stairs => {
            draw_triangle(
                vec2(center_x, center_y - radius),
                vec2(center_x - radius, center_y + radius),
                vec2(center_x + radius, center_y + radius),
                Color::from_rgba(118, 198, 178, 255),
            );
        }
        TowerMapObjectKind::Exit => {
            draw_rectangle(
                center_x - radius * 0.8,
                center_y - radius,
                radius * 1.6,
                radius * 2.0,
                Color::from_rgba(95, 162, 95, 255),
            );
            draw_rectangle_lines(
                center_x - radius * 0.8,
                center_y - radius,
                radius * 1.6,
                radius * 2.0,
                1.5,
                Color::from_rgba(206, 236, 180, 255),
            );
        }
    }
}

fn draw_player(map: &TowerMapState, origin_x: f32, origin_y: f32, tile_size: f32) {
    let center_x = origin_x + map.player_x as f32 * tile_size + tile_size * 0.5;
    let center_y = origin_y + map.player_y as f32 * tile_size + tile_size * 0.5;
    draw_circle(
        center_x,
        center_y,
        tile_size * 0.38,
        Color::from_rgba(238, 241, 213, 255),
    );
    draw_circle_lines(
        center_x,
        center_y,
        tile_size * 0.42,
        2.0,
        Color::from_rgba(45, 79, 69, 255),
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

    for (index, (label, color)) in entries.iter().enumerate() {
        let item_x = x + index as f32 * 96.0;
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
