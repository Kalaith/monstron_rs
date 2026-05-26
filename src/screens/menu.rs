use macroquad::prelude::*;

use crate::ui;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    NewGame,
    LoadGame,
    Settings,
    ExitGame,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsAction {
    ToggleFullscreen,
    Back,
}

pub fn handle_input(has_save: bool) -> Option<MenuAction> {
    if is_key_pressed(KeyCode::Enter) {
        return Some(MenuAction::NewGame);
    }

    if has_save && is_key_pressed(KeyCode::L) {
        return Some(MenuAction::LoadGame);
    }
    if is_key_pressed(KeyCode::S) {
        return Some(MenuAction::Settings);
    }

    let new_game = new_game_rect();
    if ui::button_clicked(new_game, true) {
        return Some(MenuAction::NewGame);
    }

    let load_game = load_game_rect();
    if ui::button_clicked(load_game, has_save) {
        return Some(MenuAction::LoadGame);
    }

    let settings = settings_rect();
    if ui::button_clicked(settings, true) {
        return Some(MenuAction::Settings);
    }

    let exit_game = exit_game_rect();
    if ui::button_clicked(exit_game, true) {
        return Some(MenuAction::ExitGame);
    }

    None
}

pub fn handle_settings_input() -> Option<SettingsAction> {
    if is_key_pressed(KeyCode::Escape) {
        return Some(SettingsAction::Back);
    }
    if is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::Space)
        || is_key_pressed(KeyCode::F)
        || ui::button_clicked(fullscreen_toggle_rect(), true)
    {
        return Some(SettingsAction::ToggleFullscreen);
    }
    if ui::button_clicked(settings_back_rect(), true) {
        return Some(SettingsAction::Back);
    }

    None
}

pub fn draw(title_texture: &Texture2D, has_save: bool) {
    draw_title_art(title_texture);
    ui::draw_title_button(new_game_rect(), "New Game", true);
    ui::draw_title_button(load_game_rect(), "Load Game", has_save);
    ui::draw_title_button(settings_rect(), "Settings", true);
    ui::draw_title_button(exit_game_rect(), "Exit Game", true);
}

pub fn draw_settings(fullscreen_enabled: bool) {
    draw_rectangle(0.0, 0.0, ui::VIEW_WIDTH, ui::VIEW_HEIGHT, ui::BACKGROUND);
    draw_text_ex(
        "Settings",
        72.0,
        96.0,
        TextParams {
            font_size: 44,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    ui::draw_toggle(fullscreen_toggle_rect(), "Fullscreen", fullscreen_enabled);
    ui::draw_title_button(settings_back_rect(), "Back", true);
}

fn draw_title_art(title_texture: &Texture2D) {
    let texture_size = title_texture.size();
    let scale = (ui::VIEW_WIDTH / texture_size.x).max(ui::VIEW_HEIGHT / texture_size.y);
    let width = texture_size.x * scale;
    let height = texture_size.y * scale;
    let x = (ui::VIEW_WIDTH - width) * 0.5;
    let y = (ui::VIEW_HEIGHT - height) * 0.5;

    draw_texture_ex(
        title_texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            ..Default::default()
        },
    );
}

fn new_game_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH * 0.5 - 120.0, 442.0, 240.0, 44.0)
}

fn load_game_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH * 0.5 - 120.0, 496.0, 240.0, 44.0)
}

fn settings_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH * 0.5 - 120.0, 550.0, 240.0, 44.0)
}

fn exit_game_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH * 0.5 - 120.0, 604.0, 240.0, 44.0)
}

fn fullscreen_toggle_rect() -> Rect {
    Rect::new(410.0, 278.0, 460.0, 64.0)
}

fn settings_back_rect() -> Rect {
    Rect::new(520.0, 392.0, 240.0, 44.0)
}
