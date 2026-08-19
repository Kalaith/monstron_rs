use macroquad::prelude::*;

mod map_view;

use crate::assets;
use crate::data::GameData;
use crate::engine::town_engine;
use crate::state::{GameState, TowerMapObject, TowerMapObjectKind, TowerRunState};
use crate::ui;
use macroquad_toolkit::ui::draw_ui_text_ex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TowerAction {
    Move(i32, i32),
    TapMove(i32, i32),
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
        if let Some(action) = map_view::world_tap_action(state.tower_run.as_ref().unwrap()) {
            return Some(action);
        }
        if ui::button_clicked(Rect::new(738.0, 650.0, 180.0, 58.0), true) {
            return Some(TowerAction::ReturnToTown);
        }
        if ui::button_clicked(return_button_rect(), true) {
            return Some(TowerAction::ReturnToTown);
        }
    }

    None
}

pub fn draw(state: &GameState, data: &GameData, status_message: &str) {
    if let Some(run) = &state.tower_run {
        map_view::draw_map_world(state, data, run);
        draw_run_overlay(state, data, run);
        draw_party_rail(state);
        draw_action_dock();
        draw_context_drawer(data, run);
        draw_journal(run);
        ui::draw_status_at(status_message, Rect::new(250.0, 60.0, 780.0, 28.0));
    } else {
        draw_backdrop();
        draw_header(state);
        draw_empty_run();
        draw_floor_reference(state, data);
    }

    if state.tower_run.is_none() {
        ui::draw_status(status_message);
    }
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
    draw_ui_text_ex(
        "Tower Map",
        58.0,
        72.0,
        TextParams {
            font_size: 36,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_ui_text_ex(
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

fn draw_run_overlay(state: &GameState, data: &GameData, run: &TowerRunState) {
    let floor = data.tower_floor(run.current_floor);
    let floor_name = floor
        .map(|floor| floor.name.as_str())
        .unwrap_or("Unknown Floor");
    let top = Rect::new(0.0, 0.0, ui::VIEW_WIDTH, 54.0);
    draw_overlay_panel(top);
    draw_ui_text_ex(
        "HATCHSPIRE",
        22.0,
        35.0,
        TextParams {
            font_size: 27,
            color: gold_bright(),
            ..Default::default()
        },
    );
    draw_line(205.0, 12.0, 205.0, 43.0, 1.0, gold_dim());
    draw_ui_text_ex(
        &format!(
            "FLOOR {}  ·  {}",
            run.current_floor,
            floor_name.to_uppercase()
        ),
        238.0,
        34.0,
        TextParams {
            font_size: 23,
            color: gold_bright(),
            ..Default::default()
        },
    );
    draw_ui_text_ex(
        &format!(
            "Eggs {}/{}     Supplies {}     Coins {}",
            state.egg_inventory.eggs.len() + run.found_eggs.len(),
            town_engine::egg_capacity(state),
            run.cargo_amount(),
            state.resources.amount("coins")
        ),
        755.0,
        34.0,
        TextParams {
            font_size: 19,
            color: gold_bright(),
            ..Default::default()
        },
    );
    draw_ui_text_ex(
        "SET",
        1233.0,
        37.0,
        TextParams {
            font_size: 16,
            color: gold_bright(),
            ..Default::default()
        },
    );
}

fn draw_party_rail(state: &GameState) {
    let panel = Rect::new(10.0, 68.0, 152.0, 260.0);
    draw_overlay_panel(panel);
    for (index, slot) in state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .take(3)
        .enumerate()
    {
        let Some(monster) = state.monster_roster.monster(*slot) else {
            continue;
        };
        let y = panel.y + 9.0 + index as f32 * 82.0;
        draw_rectangle(
            panel.x + 12.0,
            y,
            panel.w - 24.0,
            74.0,
            Color::from_rgba(8, 12, 14, 205),
        );
        assets::draw_party_portrait(&monster.species_id, panel.x + 8.0, y + 7.0, 61.0);
        draw_ui_text_ex(
            &monster.name,
            panel.x + 72.0,
            y + 27.0,
            TextParams {
                font_size: 17,
                color: ui::TEXT_BRIGHT,
                ..Default::default()
            },
        );
        draw_rectangle(
            panel.x + 72.0,
            y + 38.0,
            68.0,
            7.0,
            Color::from_rgba(31, 47, 42, 255),
        );
        draw_rectangle(
            panel.x + 72.0,
            y + 38.0,
            68.0 * (monster.hp.max(0) as f32 / monster.max_hp.max(1) as f32),
            7.0,
            Color::from_rgba(112, 184, 108, 255),
        );
        draw_ui_text_ex(
            &format!("HP {}/{}", monster.hp, monster.max_hp),
            panel.x + 72.0,
            y + 62.0,
            TextParams {
                font_size: 13,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    }
}

fn draw_context_drawer(data: &GameData, run: &TowerRunState) {
    let panel = Rect::new(1060.0, 104.0, 208.0, 366.0);
    draw_overlay_panel(panel);
    let focus = nearest_visible_object(run);
    let floor_name = data
        .tower_floor(run.current_floor)
        .map(|floor| floor.name.as_str())
        .unwrap_or("Dungeon");
    let name = focus
        .and_then(|object| object_name(data, object))
        .unwrap_or(floor_name);
    draw_ui_text_ex(
        name,
        panel.x + 16.0,
        panel.y + 35.0,
        TextParams {
            font_size: 24,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_ui_text_ex(
        focus
            .map(object_kind_label)
            .unwrap_or("EXPEDITION  ·  ROUTE"),
        panel.x + 16.0,
        panel.y + 65.0,
        TextParams {
            font_size: 15,
            color: gold_bright(),
            ..Default::default()
        },
    );
    if let Some(object) = focus {
        draw_context_art(data, object, panel.x + 20.0, panel.y + 78.0, 168.0, 150.0);
    } else {
        assets::draw_room_vignette(
            run.current_floor,
            panel.x + 20.0,
            panel.y + 78.0,
            168.0,
            150.0,
        );
    }
    let detail = focus
        .and_then(|object| object_detail(data, object))
        .unwrap_or("Move through lit rooms to reveal what the tower is hiding.");
    draw_wrapped_line(detail, panel.x + 16.0, panel.y + 258.0, 27, ui::TEXT_DIM);
    draw_ui_text_ex(
        if focus.is_some() {
            "Approach to interact"
        } else {
            "Explore the map"
        },
        panel.x + 48.0,
        panel.y + 348.0,
        TextParams {
            font_size: 14,
            color: ui::TEXT_DIM,
            ..Default::default()
        },
    );
}

fn nearest_visible_object(run: &TowerRunState) -> Option<&TowerMapObject> {
    run.map
        .objects
        .iter()
        .filter(|object| run.map.is_visible(object.x, object.y))
        .min_by_key(|object| {
            run.map.player_x.abs_diff(object.x) + run.map.player_y.abs_diff(object.y)
        })
}

fn object_name<'a>(data: &'a GameData, object: &'a TowerMapObject) -> Option<&'a str> {
    match object.kind {
        TowerMapObjectKind::Loot => Some(data.resource_name(&object.resource_id)),
        TowerMapObjectKind::Egg => data
            .egg_type(&object.egg_type_id)
            .map(|egg| egg.name.as_str()),
        TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss => data
            .enemy(&object.enemy_id)
            .map(|enemy| enemy.name.as_str()),
        TowerMapObjectKind::SpecialLocation => data
            .tower_special_location(&object.special_location_id)
            .map(|location| location.name.as_str()),
        TowerMapObjectKind::Stairs => Some("Deeper Stair"),
        TowerMapObjectKind::Exit => Some("Return Threshold"),
    }
}

fn object_detail<'a>(data: &'a GameData, object: &'a TowerMapObject) -> Option<&'a str> {
    match object.kind {
        TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss => data
            .enemy(&object.enemy_id)
            .map(|enemy| enemy.description.as_str()),
        TowerMapObjectKind::SpecialLocation => data
            .tower_special_location(&object.special_location_id)
            .map(|location| location.description.as_str()),
        TowerMapObjectKind::Egg => Some("A living egg waits in a tower nest."),
        TowerMapObjectKind::Loot => Some("Supplies can be carried safely back to town."),
        TowerMapObjectKind::Stairs => Some("This route leads to the next floor."),
        TowerMapObjectKind::Exit => Some("This threshold returns the party and its cargo to town."),
    }
}

fn object_kind_label(object: &TowerMapObject) -> &'static str {
    match object.kind {
        TowerMapObjectKind::Loot => "CACHE  ·  SUPPLIES",
        TowerMapObjectKind::Egg => "NEST  ·  EGG",
        TowerMapObjectKind::Enemy => "ENCOUNTER  ·  WANDERING",
        TowerMapObjectKind::Boss => "ENCOUNTER  ·  BOSS",
        TowerMapObjectKind::SpecialLocation => "LANDMARK  ·  EVENT",
        TowerMapObjectKind::Stairs => "ROUTE  ·  DESCENT",
        TowerMapObjectKind::Exit => "ROUTE  ·  RETURN",
    }
}

fn draw_context_art(data: &GameData, object: &TowerMapObject, x: f32, y: f32, w: f32, h: f32) {
    match object.kind {
        TowerMapObjectKind::Loot => assets::draw_dungeon_feature(1, x, y, w, h),
        TowerMapObjectKind::Egg => assets::draw_egg_badge(&object.egg_type_id, x + 18.0, y, h),
        TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss => {
            assets::draw_dungeon_enemy_by_id(&object.enemy_id, x, y, w, h)
        }
        TowerMapObjectKind::SpecialLocation => {
            if let Some(location) = data.tower_special_location(&object.special_location_id) {
                assets::draw_special_location(location.visual, x, y, w, h);
            }
        }
        TowerMapObjectKind::Stairs => assets::draw_dungeon_feature(2, x, y, w, h),
        TowerMapObjectKind::Exit => assets::draw_dungeon_feature(3, x, y, w, h),
    }
}

fn draw_action_dock() {
    for (index, (icon, title)) in [("+", "Explore"), ("*", "Camp"), ("<", "Retreat")]
        .iter()
        .enumerate()
    {
        let rect = Rect::new(350.0 + index as f32 * 194.0, 650.0, 180.0, 58.0);
        draw_overlay_panel(rect);
        draw_ui_text_ex(
            icon,
            rect.x + 20.0,
            rect.y + 39.0,
            TextParams {
                font_size: 29,
                color: gold_bright(),
                ..Default::default()
            },
        );
        draw_ui_text_ex(
            title,
            rect.x + 62.0,
            rect.y + 37.0,
            TextParams {
                font_size: 22,
                color: gold_bright(),
                ..Default::default()
            },
        );
    }
}

fn draw_journal(run: &TowerRunState) {
    let panel = Rect::new(18.0, 538.0, 218.0, 148.0);
    draw_overlay_panel(panel);
    draw_ui_text_ex(
        "EXPEDITION JOURNAL",
        panel.x + 16.0,
        panel.y + 24.0,
        TextParams {
            font_size: 16,
            color: gold_bright(),
            ..Default::default()
        },
    );
    for (index, message) in run.event_log.iter().rev().take(2).enumerate() {
        draw_ui_text_ex(
            &format!("• {}", message),
            panel.x + 16.0,
            panel.y + 78.0 + index as f32 * 24.0,
            TextParams {
                font_size: 12,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    }
}

fn draw_overlay_panel(rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(8, 12, 13, 238),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        Color::from_rgba(104, 76, 37, 235),
    );
    draw_rectangle_lines(
        rect.x + 4.0,
        rect.y + 4.0,
        rect.w - 8.0,
        rect.h - 8.0,
        1.0,
        Color::from_rgba(184, 133, 60, 100),
    );
}

fn gold_bright() -> Color {
    Color::from_rgba(227, 196, 139, 255)
}
fn gold_dim() -> Color {
    Color::from_rgba(112, 82, 42, 220)
}

fn draw_empty_run() {
    let rect = Rect::new(32.0, 124.0, 560.0, 476.0);
    ui::draw_panel(rect);
    ui::draw_section_title("No Active Run", rect.x + 20.0, rect.y + 34.0);
    draw_ui_text_ex(
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
        draw_ui_text_ex(
            &format!("{}  {}", floor.floor, floor.name),
            rect.x + 20.0,
            y,
            TextParams {
                font_size: 19,
                color,
                ..Default::default()
            },
        );
        draw_ui_text_ex(
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
            draw_ui_text_ex(
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
        draw_ui_text_ex(
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

fn town_button_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH - 116.0, 18.0, 98.0, 40.0)
}

fn return_button_rect() -> Rect {
    town_button_rect()
}
