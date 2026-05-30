use macroquad::prelude::*;

use crate::assets;
use crate::data::GameData;
use crate::engine::{tower_engine, town_engine};
use crate::state::{
    GameState, TowerMapObject, TowerMapObjectKind, TowerMapState, TowerRunState, TowerTileKind,
    TowerTileVisibility,
};
use crate::ui;

const VIEWPORT_TILES_W: u32 = 17;
const VIEWPORT_TILES_H: u32 = 13;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TowerAction {
    Move(i32, i32),
    ReturnToTown,
    ToTown,
}

pub fn handle_input(state: &GameState) -> Option<TowerAction> {
    if is_key_pressed(KeyCode::Escape) {
        return if state.tower_run.is_some() {
            Some(TowerAction::ReturnToTown)
        } else {
            Some(TowerAction::ToTown)
        };
    }

    if state.tower_run.is_some() {
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            return Some(TowerAction::Move(0, -1));
        }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
            return Some(TowerAction::Move(0, 1));
        }
        if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            return Some(TowerAction::Move(-1, 0));
        }
        if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
            return Some(TowerAction::Move(1, 0));
        }
        if is_key_pressed(KeyCode::R) {
            return Some(TowerAction::ReturnToTown);
        }
    }

    if ui::button_clicked(town_button_rect(), true) {
        return if state.tower_run.is_some() {
            Some(TowerAction::ReturnToTown)
        } else {
            Some(TowerAction::ToTown)
        };
    }

    if state.tower_run.is_some() {
        for (action, rect) in movement_buttons() {
            if ui::button_clicked(rect, true) {
                return Some(action);
            }
        }
        if ui::button_clicked(return_button_rect(), true) {
            return Some(TowerAction::ReturnToTown);
        }
    }

    None
}

pub fn draw(state: &GameState, data: &GameData, status_message: &str) {
    draw_backdrop();
    draw_header(state);

    if let Some(run) = &state.tower_run {
        draw_map_panel(run);
        draw_run_sidebar(state, data, run);
    } else {
        draw_empty_run();
        draw_floor_reference(state, data);
    }

    ui::draw_status(status_message);
}

fn draw_backdrop() {
    draw_rectangle(
        0.0,
        0.0,
        ui::VIEW_WIDTH,
        ui::VIEW_HEIGHT,
        Color::from_rgba(12, 16, 18, 255),
    );
    draw_rectangle(
        0.0,
        520.0,
        ui::VIEW_WIDTH,
        200.0,
        Color::from_rgba(26, 38, 34, 255),
    );

    for index in 0..8 {
        let x = 82.0 + index as f32 * 156.0;
        let height = 270.0 + (index % 4) as f32 * 54.0;
        draw_rectangle(
            x,
            520.0 - height,
            86.0,
            height,
            Color::from_rgba(34, 42, 49, 210),
        );
        draw_rectangle_lines(
            x,
            520.0 - height,
            86.0,
            height,
            2.0,
            Color::from_rgba(65, 80, 83, 190),
        );
    }
}

fn draw_header(state: &GameState) {
    ui::draw_panel(Rect::new(32.0, 24.0, ui::VIEW_WIDTH - 64.0, 78.0));
    draw_text_ex(
        "Tower Map",
        58.0,
        72.0,
        TextParams {
            font_size: 36,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!(
            "Best {}  Unlocked {}",
            state.tower_progress.best_floor, state.tower_progress.unlocked_floor
        ),
        760.0,
        70.0,
        TextParams {
            font_size: 24,
            color: ui::ACCENT,
            ..Default::default()
        },
    );
    let label = if state.tower_run.is_some() {
        "Return"
    } else {
        "Town"
    };
    ui::draw_button(town_button_rect(), label, true);
}

fn draw_map_panel(run: &TowerRunState) {
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
        draw_text_ex(
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

fn draw_run_sidebar(state: &GameState, data: &GameData, run: &TowerRunState) {
    let rect = Rect::new(832.0, 124.0, 416.0, 494.0);
    ui::draw_panel(rect);

    let floor = data.tower_floor(run.current_floor);
    let floor_name = floor
        .map(|floor| floor.name.as_str())
        .unwrap_or("Unknown Floor");
    let theme = floor
        .map(|floor| floor.theme.as_str())
        .unwrap_or("The tower records are missing.");

    ui::draw_section_title("Current Floor", rect.x + 20.0, rect.y + 34.0);
    draw_text_ex(
        &format!("Floor {}: {}", run.current_floor, floor_name),
        rect.x + 20.0,
        rect.y + 72.0,
        TextParams {
            font_size: 21,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_wrapped_line(theme, rect.x + 20.0, rect.y + 104.0, 48, ui::TEXT_DIM);

    draw_text_ex(
        &format!(
            "{}  Steps {}  Party {}  Ready {}",
            run.goal,
            run.rooms_explored,
            tower_engine::party_count(state),
            tower_engine::battle_ready_party_count(state)
        ),
        rect.x + 20.0,
        rect.y + 176.0,
        TextParams {
            font_size: 18,
            color: ui::TEXT,
            ..Default::default()
        },
    );

    draw_cargo_summary(
        state,
        data,
        run,
        rect.x + 20.0,
        rect.y + 214.0,
        rect.w - 40.0,
    );
    draw_events(run, rect.x + 20.0, rect.y + 366.0, rect.w - 40.0);
    ui::draw_button(return_button_rect(), "Return", true);
}

fn draw_cargo_summary(
    state: &GameState,
    data: &GameData,
    run: &TowerRunState,
    x: f32,
    y: f32,
    width: f32,
) {
    draw_text_ex(
        &format!("Run Loot: {} item(s)", run.cargo_amount()),
        x,
        y,
        TextParams {
            font_size: 18,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );

    if run.cargo.is_empty() {
        draw_text_ex(
            "No materials collected yet.",
            x,
            y + 28.0,
            TextParams {
                font_size: 16,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    } else {
        for (index, stack) in run.cargo.iter().take(3).enumerate() {
            draw_text_ex(
                &format!(
                    "{} {}",
                    stack.amount,
                    data.resource_name(&stack.resource_id)
                ),
                x,
                y + 28.0 + index as f32 * 22.0,
                TextParams {
                    font_size: 16,
                    color: ui::TEXT,
                    ..Default::default()
                },
            );
        }
    }

    draw_text_ex(
        &format!(
            "Egg slots: {}/{}",
            state.egg_inventory.eggs.len() + run.found_eggs.len(),
            town_engine::egg_capacity(state)
        ),
        x + width * 0.52,
        y,
        TextParams {
            font_size: 16,
            color: ui::TEXT_DIM,
            ..Default::default()
        },
    );

    if run.found_eggs.is_empty() {
        draw_text_ex(
            "No eggs found.",
            x + width * 0.52,
            y + 28.0,
            TextParams {
                font_size: 16,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    } else {
        for (index, egg) in run.found_eggs.iter().take(3).enumerate() {
            let egg_y = y + 20.0 + index as f32 * 32.0;
            assets::draw_egg_badge(egg.palette_seed, x + width * 0.52, egg_y, 24.0);
            let egg_name = data
                .egg_type(&egg.egg_type_id)
                .map(|egg_type| egg_type.name.as_str())
                .unwrap_or(egg.egg_type_id.as_str());
            draw_text_ex(
                egg_name,
                x + width * 0.52 + 32.0,
                egg_y + 20.0,
                TextParams {
                    font_size: 15,
                    color: ui::TEXT,
                    ..Default::default()
                },
            );
        }
    }
}

fn draw_events(run: &TowerRunState, x: f32, y: f32, width: f32) {
    draw_text_ex(
        "Recent Events",
        x,
        y,
        TextParams {
            font_size: 18,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    for (index, message) in run.event_log.iter().rev().take(4).enumerate() {
        draw_wrapped_line(message, x, y + 30.0 + index as f32 * 44.0, 48, ui::TEXT);
    }
    draw_rectangle_lines(x - 2.0, y + 14.0, width + 4.0, 122.0, 1.0, ui::PANEL_EDGE);
}

fn draw_empty_run() {
    let rect = Rect::new(32.0, 124.0, 560.0, 476.0);
    ui::draw_panel(rect);
    ui::draw_section_title("No Active Run", rect.x + 20.0, rect.y + 34.0);
    draw_text_ex(
        "Enter the tower from town to begin a run.",
        rect.x + 20.0,
        rect.y + 88.0,
        TextParams {
            font_size: 25,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_wrapped_line(
        "Dungeon maps are generated each run with rooms, corridors, caches, eggs, enemies, stairs, and exits.",
        rect.x + 20.0,
        rect.y + 132.0,
        58,
        ui::TEXT_DIM,
    );
}

fn draw_floor_reference(state: &GameState, data: &GameData) {
    let rect = Rect::new(620.0, 124.0, 628.0, 476.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Known Floors", rect.x + 20.0, rect.y + 34.0);

    for (index, floor) in data.tower_floors.iter().take(10).enumerate() {
        let y = rect.y + 76.0 + index as f32 * 38.0;
        let color = if floor.floor <= state.tower_progress.unlocked_floor {
            ui::TEXT_BRIGHT
        } else {
            ui::TEXT_DIM
        };
        draw_text_ex(
            &format!("{}  {}", floor.floor, floor.name),
            rect.x + 20.0,
            y,
            TextParams {
                font_size: 19,
                color,
                ..Default::default()
            },
        );
        draw_text_ex(
            &format!("{}  {}", floor.theme, floor.enemy_hint),
            rect.x + 300.0,
            y,
            TextParams {
                font_size: 16,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    }
}

fn draw_wrapped_line(text: &str, x: f32, y: f32, max_chars: usize, color: Color) {
    let mut line = String::new();
    let mut row = 0;

    for word in text.split_whitespace() {
        let next_len = if line.is_empty() {
            word.len()
        } else {
            line.len() + 1 + word.len()
        };
        if next_len > max_chars && !line.is_empty() {
            draw_text_ex(
                &line,
                x,
                y + row as f32 * 20.0,
                TextParams {
                    font_size: 16,
                    color,
                    ..Default::default()
                },
            );
            line.clear();
            row += 1;
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }

    if !line.is_empty() {
        draw_text_ex(
            &line,
            x,
            y + row as f32 * 20.0,
            TextParams {
                font_size: 16,
                color,
                ..Default::default()
            },
        );
    }
}

fn movement_buttons() -> [(TowerAction, Rect); 4] {
    [
        (
            TowerAction::Move(0, -1),
            Rect::new(688.0, 532.0, 42.0, 34.0),
        ),
        (
            TowerAction::Move(-1, 0),
            Rect::new(640.0, 568.0, 42.0, 34.0),
        ),
        (TowerAction::Move(1, 0), Rect::new(736.0, 568.0, 42.0, 34.0)),
        (TowerAction::Move(0, 1), Rect::new(688.0, 568.0, 42.0, 34.0)),
    ]
}

fn town_button_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH - 148.0, 44.0, 86.0, 34.0)
}

fn return_button_rect() -> Rect {
    Rect::new(1100.0, 566.0, 118.0, 34.0)
}
