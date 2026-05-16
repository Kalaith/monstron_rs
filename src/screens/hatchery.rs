use macroquad::prelude::*;

use crate::assets;
use crate::data::GameData;
use crate::state::GameState;
use crate::ui;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HatcheryAction {
    ToTown,
    DiscoverEgg,
    WarmEgg(u64),
    HatchEgg(u64),
}

pub fn handle_input(state: &GameState) -> Option<HatcheryAction> {
    if is_key_pressed(KeyCode::Escape) {
        return Some(HatcheryAction::ToTown);
    }
    if is_key_pressed(KeyCode::E) {
        return Some(HatcheryAction::DiscoverEgg);
    }

    if ui::button_clicked(town_button_rect(), true) {
        return Some(HatcheryAction::ToTown);
    }
    if ui::button_clicked(discover_button_rect(), true) {
        return Some(HatcheryAction::DiscoverEgg);
    }

    for (index, egg) in state.egg_inventory.eggs.iter().take(6).enumerate() {
        let warm_enabled = egg.days_remaining > 0;
        if ui::button_clicked(warm_button_rect(index), warm_enabled) {
            return Some(HatcheryAction::WarmEgg(egg.id));
        }

        let hatch_enabled = egg.days_remaining == 0;
        if ui::button_clicked(hatch_button_rect(index), hatch_enabled) {
            return Some(HatcheryAction::HatchEgg(egg.id));
        }
    }

    None
}

pub fn draw(state: &GameState, data: &GameData, status_message: &str) {
    draw_backdrop();
    draw_header(state);
    draw_egg_inventory(state, data);
    draw_reference(data);
    ui::draw_status(status_message);
}

fn draw_backdrop() {
    draw_rectangle(
        0.0,
        0.0,
        ui::VIEW_WIDTH,
        ui::VIEW_HEIGHT,
        Color::from_rgba(24, 28, 32, 255),
    );
    draw_circle(1120.0, 130.0, 100.0, Color::from_rgba(129, 187, 138, 30));
    draw_rectangle(
        0.0,
        500.0,
        ui::VIEW_WIDTH,
        220.0,
        Color::from_rgba(40, 50, 42, 255),
    );
}

fn draw_header(state: &GameState) {
    ui::draw_panel(Rect::new(32.0, 24.0, ui::VIEW_WIDTH - 64.0, 78.0));
    draw_text_ex(
        "Hatchery",
        58.0,
        72.0,
        TextParams {
            font_size: 36,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("Day {}  Eggs {}", state.day, state.egg_inventory.eggs.len()),
        778.0,
        70.0,
        TextParams {
            font_size: 24,
            color: ui::ACCENT,
            ..Default::default()
        },
    );
    ui::draw_button(town_button_rect(), "Town", true);
}

fn draw_egg_inventory(state: &GameState, data: &GameData) {
    let rect = Rect::new(32.0, 124.0, 780.0, 476.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Egg Inventory", rect.x + 20.0, rect.y + 34.0);
    ui::draw_button(discover_button_rect(), "Recover Egg", true);

    if state.egg_inventory.eggs.is_empty() {
        draw_text_ex(
            "No eggs are waiting. Recover one from the tower edge.",
            rect.x + 24.0,
            rect.y + 108.0,
            TextParams {
                font_size: 24,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
        return;
    }

    for (index, egg) in state.egg_inventory.eggs.iter().take(6).enumerate() {
        let y = rect.y + 78.0 + index as f32 * 60.0;
        let egg_type = data.egg_type(&egg.egg_type_id);
        let name = egg_type
            .map(|egg_type| egg_type.name.as_str())
            .unwrap_or(egg.egg_type_id.as_str());
        let rarity = egg_type
            .map(|egg_type| egg_type.rarity.as_str())
            .unwrap_or("?");
        assets::draw_egg_badge(egg.palette_seed, rect.x + 24.0, y - 34.0, 42.0);
        draw_text_ex(
            &format!("{} #{}", name, egg.id),
            rect.x + 82.0,
            y,
            TextParams {
                font_size: 22,
                color: ui::TEXT_BRIGHT,
                ..Default::default()
            },
        );
        draw_text_ex(
            &format!(
                "{}  Floor {}  {} day(s) remaining",
                rarity, egg.origin_floor, egg.days_remaining
            ),
            rect.x + 82.0,
            y + 24.0,
            TextParams {
                font_size: 17,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
        ui::draw_button(warm_button_rect(index), "Warm", egg.days_remaining > 0);
        ui::draw_button(hatch_button_rect(index), "Hatch", egg.days_remaining == 0);
    }
}

fn draw_reference(data: &GameData) {
    let rect = Rect::new(836.0, 124.0, ui::VIEW_WIDTH - 868.0, 476.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Known Egg Types", rect.x + 20.0, rect.y + 34.0);

    for (index, egg_type) in data.egg_types.iter().take(8).enumerate() {
        let y = rect.y + 76.0 + index as f32 * 45.0;
        draw_text_ex(
            &egg_type.name,
            rect.x + 20.0,
            y,
            TextParams {
                font_size: 19,
                color: ui::TEXT_BRIGHT,
                ..Default::default()
            },
        );
        draw_text_ex(
            &format!(
                "{}  Floor {}  Hatch {}d",
                egg_type.rarity, egg_type.discovery_floor, egg_type.hatch_days
            ),
            rect.x + 20.0,
            y + 20.0,
            TextParams {
                font_size: 15,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    }
}

fn town_button_rect() -> Rect {
    Rect::new(ui::VIEW_WIDTH - 148.0, 44.0, 86.0, 34.0)
}

fn discover_button_rect() -> Rect {
    Rect::new(946.0, 44.0, 154.0, 34.0)
}

fn warm_button_rect(index: usize) -> Rect {
    Rect::new(596.0, 166.0 + index as f32 * 60.0, 82.0, 30.0)
}

fn hatch_button_rect(index: usize) -> Rect {
    Rect::new(688.0, 166.0 + index as f32 * 60.0, 82.0, 30.0)
}
