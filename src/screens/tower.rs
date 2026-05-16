use macroquad::prelude::*;

use crate::assets;
use crate::data::GameData;
use crate::engine::tower_engine;
use crate::state::{GameState, TowerRunState};
use crate::ui;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TowerAction {
    Explore,
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

    if state.tower_run.is_some()
        && (is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter))
    {
        return Some(TowerAction::Explore);
    }

    if state.tower_run.is_some() && is_key_pressed(KeyCode::R) {
        return Some(TowerAction::ReturnToTown);
    }

    if ui::button_clicked(town_button_rect(), true) {
        return if state.tower_run.is_some() {
            Some(TowerAction::ReturnToTown)
        } else {
            Some(TowerAction::ToTown)
        };
    }

    if let Some(run) = &state.tower_run {
        if ui::button_clicked(explore_button_rect(), run.pressure < run.pressure_limit) {
            return Some(TowerAction::Explore);
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
        draw_run_overview(state, data, run);
        draw_cargo(data, run);
        draw_events(run);
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
        Color::from_rgba(16, 21, 25, 255),
    );
    draw_rectangle(
        0.0,
        520.0,
        ui::VIEW_WIDTH,
        200.0,
        Color::from_rgba(30, 42, 38, 255),
    );
    draw_circle(1040.0, 118.0, 128.0, Color::from_rgba(130, 170, 120, 28));

    for index in 0..7 {
        let x = 120.0 + index as f32 * 160.0;
        let height = 310.0 + (index % 3) as f32 * 62.0;
        draw_rectangle(
            x,
            520.0 - height,
            94.0,
            height,
            Color::from_rgba(40, 47, 55, 210),
        );
        draw_rectangle_lines(
            x,
            520.0 - height,
            94.0,
            height,
            2.0,
            Color::from_rgba(79, 91, 101, 200),
        );
    }
}

fn draw_header(state: &GameState) {
    ui::draw_panel(Rect::new(32.0, 24.0, ui::VIEW_WIDTH - 64.0, 78.0));
    draw_text_ex(
        "Tower Exploration",
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

fn draw_run_overview(state: &GameState, data: &GameData, run: &TowerRunState) {
    let rect = Rect::new(32.0, 124.0, 390.0, 476.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Current Floor", rect.x + 20.0, rect.y + 34.0);

    let floor = data.tower_floor(run.current_floor);
    let floor_name = floor
        .map(|floor| floor.name.as_str())
        .unwrap_or("Unknown Floor");
    let theme = floor
        .map(|floor| floor.theme.as_str())
        .unwrap_or("The tower records are missing.");

    draw_text_ex(
        &format!("Floor {}: {}", run.current_floor, floor_name),
        rect.x + 20.0,
        rect.y + 78.0,
        TextParams {
            font_size: 24,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_wrapped_line(theme, rect.x + 20.0, rect.y + 114.0, 44, ui::TEXT_DIM);
    draw_text_ex(
        &format!(
            "Rooms {}  Party {}  Ready {}",
            run.rooms_explored,
            tower_engine::party_count(state),
            tower_engine::battle_ready_party_count(state)
        ),
        rect.x + 20.0,
        rect.y + 186.0,
        TextParams {
            font_size: 20,
            color: ui::TEXT,
            ..Default::default()
        },
    );

    draw_pressure_bar(run, rect.x + 20.0, rect.y + 220.0, rect.w - 40.0);

    draw_wrapped_line(
        "Pressure rises with each room and enemy patrol. Return before it maxes.",
        rect.x + 20.0,
        rect.y + 292.0,
        43,
        ui::TEXT_DIM,
    );

    ui::draw_button(
        explore_button_rect(),
        "Explore Room",
        run.pressure < run.pressure_limit,
    );
    ui::draw_button(return_button_rect(), "Return", true);
}

fn draw_pressure_bar(run: &TowerRunState, x: f32, y: f32, width: f32) {
    draw_text_ex(
        &format!("Pressure {}/{}", run.pressure, run.pressure_limit),
        x,
        y,
        TextParams {
            font_size: 18,
            color: ui::TEXT,
            ..Default::default()
        },
    );
    let bar_y = y + 16.0;
    draw_rectangle(x, bar_y, width, 20.0, Color::from_rgba(30, 36, 40, 255));
    let ratio = if run.pressure_limit == 0 {
        0.0
    } else {
        run.pressure as f32 / run.pressure_limit as f32
    };
    let color = if ratio > 0.75 {
        Color::from_rgba(211, 99, 81, 255)
    } else {
        ui::ACCENT
    };
    draw_rectangle(x, bar_y, width * ratio.clamp(0.0, 1.0), 20.0, color);
    draw_rectangle_lines(x, bar_y, width, 20.0, 1.5, ui::PANEL_EDGE);
}

fn draw_cargo(data: &GameData, run: &TowerRunState) {
    let rect = Rect::new(444.0, 124.0, 388.0, 476.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Run Loot", rect.x + 20.0, rect.y + 34.0);

    draw_text_ex(
        &format!("Cargo items: {}", run.cargo_amount()),
        rect.x + 20.0,
        rect.y + 72.0,
        TextParams {
            font_size: 20,
            color: ui::TEXT,
            ..Default::default()
        },
    );

    if run.cargo.is_empty() {
        draw_text_ex(
            "No materials collected yet.",
            rect.x + 20.0,
            rect.y + 110.0,
            TextParams {
                font_size: 19,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    } else {
        for (index, stack) in run.cargo.iter().take(6).enumerate() {
            let y = rect.y + 112.0 + index as f32 * 30.0;
            draw_text_ex(
                &format!(
                    "{} {}",
                    stack.amount,
                    data.resource_name(&stack.resource_id)
                ),
                rect.x + 20.0,
                y,
                TextParams {
                    font_size: 20,
                    color: ui::TEXT_BRIGHT,
                    ..Default::default()
                },
            );
        }
    }

    ui::draw_section_title("Found Eggs", rect.x + 20.0, rect.y + 288.0);
    if run.found_eggs.is_empty() {
        draw_text_ex(
            "No eggs found on this run.",
            rect.x + 20.0,
            rect.y + 326.0,
            TextParams {
                font_size: 19,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
        return;
    }

    for (index, egg) in run.found_eggs.iter().take(4).enumerate() {
        let y = rect.y + 326.0 + index as f32 * 42.0;
        assets::draw_egg_badge(egg.palette_seed, rect.x + 20.0, y - 28.0, 32.0);
        let egg_name = data
            .egg_type(&egg.egg_type_id)
            .map(|egg_type| egg_type.name.as_str())
            .unwrap_or(egg.egg_type_id.as_str());
        draw_text_ex(
            egg_name,
            rect.x + 62.0,
            y,
            TextParams {
                font_size: 20,
                color: ui::TEXT_BRIGHT,
                ..Default::default()
            },
        );
        draw_text_ex(
            &format!("Floor {}  Hatch {}d", egg.origin_floor, egg.hatch_days),
            rect.x + 62.0,
            y + 20.0,
            TextParams {
                font_size: 15,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    }
}

fn draw_events(run: &TowerRunState) {
    let rect = Rect::new(854.0, 124.0, 394.0, 476.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Floor Events", rect.x + 20.0, rect.y + 34.0);

    for (index, message) in run.event_log.iter().rev().take(7).enumerate() {
        let y = rect.y + 76.0 + index as f32 * 54.0;
        draw_wrapped_line(message, rect.x + 20.0, y, 44, ui::TEXT);
    }
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
        "Exploration collects materials and eggs. Enemy events start turn-based combat.",
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
            &format!("Pressure {}  {}", floor.pressure_limit, floor.enemy_hint),
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
                    font_size: 17,
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
                font_size: 17,
                color,
                ..Default::default()
            },
        );
    }
}

fn town_button_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH - 148.0, 44.0, 86.0, 34.0)
}

fn explore_button_rect() -> Rect {
    Rect::new(72.0, 526.0, 150.0, 38.0)
}

fn return_button_rect() -> Rect {
    Rect::new(240.0, 526.0, 150.0, 38.0)
}
