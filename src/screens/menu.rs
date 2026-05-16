use macroquad::prelude::*;

use crate::ui;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    NewGame,
    LoadGame,
}

pub fn handle_input(has_save: bool) -> Option<MenuAction> {
    if is_key_pressed(KeyCode::Enter) {
        return Some(MenuAction::NewGame);
    }

    if has_save && is_key_pressed(KeyCode::L) {
        return Some(MenuAction::LoadGame);
    }

    let new_game = new_game_rect();
    if ui::button_clicked(new_game, true) {
        return Some(MenuAction::NewGame);
    }

    let load_game = load_game_rect();
    if ui::button_clicked(load_game, has_save) {
        return Some(MenuAction::LoadGame);
    }

    None
}

pub fn draw(has_save: bool, status_message: &str) {
    let title = "Hatchspire";
    let subtitle = "Raise tower-born monsters and rebuild the camp below the spire.";
    let center_x = ui::VIEW_WIDTH * 0.5;

    draw_text_ex(
        title,
        center_x - measure_text(title, None, 64, 1.0).width * 0.5,
        170.0,
        TextParams {
            font_size: 64,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_text_ex(
        subtitle,
        center_x - measure_text(subtitle, None, 24, 1.0).width * 0.5,
        215.0,
        TextParams {
            font_size: 24,
            color: ui::TEXT_DIM,
            ..Default::default()
        },
    );

    draw_tower_mark(center_x, 350.0);
    ui::draw_button(new_game_rect(), "New Save", true);
    ui::draw_button(load_game_rect(), "Load Save", has_save);

    let hint = if has_save {
        "Enter: new save   L: load save"
    } else {
        "Enter: new save"
    };
    ui::draw_centered_text(hint, center_x, 590.0, 20, ui::TEXT_DIM);
    ui::draw_status(status_message);
}

fn draw_tower_mark(center_x: f32, base_y: f32) {
    let stone = Color::from_rgba(114, 126, 139, 255);
    let shadow = Color::from_rgba(43, 50, 60, 255);
    draw_triangle(
        vec2(center_x, base_y - 135.0),
        vec2(center_x - 60.0, base_y + 40.0),
        vec2(center_x + 60.0, base_y + 40.0),
        shadow,
    );
    draw_rectangle(center_x - 34.0, base_y - 74.0, 68.0, 114.0, stone);
    draw_rectangle(center_x - 24.0, base_y - 44.0, 18.0, 28.0, ui::BACKGROUND);
    draw_rectangle(center_x + 6.0, base_y - 44.0, 18.0, 28.0, ui::BACKGROUND);
    draw_circle(center_x, base_y - 100.0, 10.0, ui::ACCENT);
}

fn new_game_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH * 0.5 - 120.0, 470.0, 240.0, 46.0)
}

fn load_game_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH * 0.5 - 120.0, 526.0, 240.0, 46.0)
}
