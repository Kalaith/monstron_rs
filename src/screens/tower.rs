use macroquad::prelude::*;

mod map_view;

use crate::data::GameData;
use crate::engine::{tower_engine, town_engine};
use crate::state::{GameState, TowerRunState};
use crate::ui;
use macroquad_toolkit::ui::draw_ui_text_ex;

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
    if let Some(run) = &state.tower_run {
        map_view::draw_map_world(run);
        draw_run_overlay(state, data, run);
    } else {
        draw_backdrop();
        draw_header(state);
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
    let panel = Rect::new(18.0, 18.0, 444.0, 116.0);
    draw_overlay_panel(panel);
    draw_ui_text_ex(
        &format!("Floor {}  {}", run.current_floor, floor_name),
        panel.x + 18.0,
        panel.y + 34.0,
        TextParams {
            font_size: 25,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_ui_text_ex(
        &format!(
            "{}  •  Steps {}  •  Party {}  •  Ready {}",
            run.goal,
            run.rooms_explored,
            tower_engine::party_count(state),
            tower_engine::battle_ready_party_count(state)
        ),
        panel.x + 18.0,
        panel.y + 64.0,
        TextParams {
            font_size: 18,
            color: ui::ACCENT,
            ..Default::default()
        },
    );
    draw_ui_text_ex(
        &format!(
            "Loot {}   Eggs {}/{}   Pressure {}/{}",
            run.cargo_amount(),
            state.egg_inventory.eggs.len() + run.found_eggs.len(),
            town_engine::egg_capacity(state),
            run.pressure,
            run.pressure_limit
        ),
        panel.x + 18.0,
        panel.y + 92.0,
        TextParams {
            font_size: 17,
            color: ui::TEXT,
            ..Default::default()
        },
    );

    if let Some(message) = run.event_log.last() {
        let event_panel = Rect::new(18.0, 144.0, 444.0, 44.0);
        draw_overlay_panel(event_panel);
        draw_ui_text_ex(
            message,
            event_panel.x + 16.0,
            event_panel.y + 30.0,
            TextParams {
                font_size: 16,
                color: ui::TEXT,
                ..Default::default()
            },
        );
    }

    ui::draw_button(town_button_rect(), "Return", true);
}

fn draw_overlay_panel(rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(12, 17, 18, 226),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, ui::PANEL_EDGE);
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

fn movement_buttons() -> [(TowerAction, Rect); 4] {
    [
        (
            TowerAction::Move(0, -1),
            Rect::new(1178.0, 568.0, 44.0, 42.0),
        ),
        (
            TowerAction::Move(-1, 0),
            Rect::new(1128.0, 616.0, 44.0, 42.0),
        ),
        (
            TowerAction::Move(1, 0),
            Rect::new(1228.0, 616.0, 44.0, 42.0),
        ),
        (
            TowerAction::Move(0, 1),
            Rect::new(1178.0, 616.0, 44.0, 42.0),
        ),
    ]
}

fn town_button_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH - 116.0, 18.0, 98.0, 40.0)
}

fn return_button_rect() -> Rect {
    town_button_rect()
}
