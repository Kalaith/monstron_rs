use macroquad::prelude::*;

mod map_view;

use crate::assets;
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
    draw_backdrop();
    draw_header(state);

    if let Some(run) = &state.tower_run {
        map_view::draw_map_panel(run);
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
    draw_ui_text_ex(
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

    draw_ui_text_ex(
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
    draw_ui_text_ex(
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
        draw_ui_text_ex(
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
            draw_ui_text_ex(
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

    draw_ui_text_ex(
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
        draw_ui_text_ex(
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
            draw_ui_text_ex(
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
    draw_ui_text_ex(
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
