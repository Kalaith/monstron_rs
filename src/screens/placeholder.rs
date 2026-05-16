use macroquad::prelude::*;

use crate::ui;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaceholderAction {
    ToTown,
    ToTower,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaceholderKind {
    DungeonPrep,
    EndOfDay,
}

pub fn handle_input(kind: PlaceholderKind) -> Option<PlaceholderAction> {
    if is_key_pressed(KeyCode::Escape) {
        return Some(PlaceholderAction::ToTown);
    }

    if kind == PlaceholderKind::DungeonPrep && is_key_pressed(KeyCode::Enter) {
        return Some(PlaceholderAction::ToTower);
    }

    if kind == PlaceholderKind::EndOfDay && is_key_pressed(KeyCode::Enter) {
        return Some(PlaceholderAction::ToTown);
    }

    for (action, rect, enabled) in buttons(kind) {
        if ui::button_clicked(rect, enabled) {
            return Some(action);
        }
    }

    None
}

pub fn draw(kind: PlaceholderKind, status_message: &str) {
    let rect = Rect::new(220.0, 150.0, ui::VIEW_WIDTH - 440.0, 360.0);
    ui::draw_panel(rect);

    let (title, body, hint) = copy(kind);
    ui::draw_centered_text(
        title,
        ui::VIEW_WIDTH * 0.5,
        rect.y + 76.0,
        38,
        ui::TEXT_BRIGHT,
    );
    ui::draw_centered_text(body, ui::VIEW_WIDTH * 0.5, rect.y + 135.0, 22, ui::TEXT);
    ui::draw_centered_text(hint, ui::VIEW_WIDTH * 0.5, rect.y + 174.0, 19, ui::TEXT_DIM);

    for (action, button_rect, enabled) in buttons(kind) {
        let label = match action {
            PlaceholderAction::ToTown => "Town",
            PlaceholderAction::ToTower => "Enter Tower",
        };
        ui::draw_button(button_rect, label, enabled);
    }

    ui::draw_status(status_message);
}

fn copy(kind: PlaceholderKind) -> (&'static str, &'static str, &'static str) {
    match kind {
        PlaceholderKind::DungeonPrep => (
            "Dungeon Prep",
            "The stable manages the six-slot party before a tower run.",
            "Enter starts tower exploration. Esc returns to town.",
        ),
        PlaceholderKind::EndOfDay => (
            "End Of Day",
            "The day has advanced and monsters have recovered.",
            "Enter returns to town.",
        ),
    }
}

fn buttons(kind: PlaceholderKind) -> Vec<(PlaceholderAction, Rect, bool)> {
    let center_x = ui::VIEW_WIDTH * 0.5;
    let y = 390.0;
    match kind {
        PlaceholderKind::DungeonPrep => vec![
            (
                PlaceholderAction::ToTower,
                Rect::new(center_x - 220.0, y, 200.0, 46.0),
                true,
            ),
            (
                PlaceholderAction::ToTown,
                Rect::new(center_x + 20.0, y, 200.0, 46.0),
                true,
            ),
        ],
        PlaceholderKind::EndOfDay => vec![(
            PlaceholderAction::ToTown,
            Rect::new(center_x - 100.0, y, 200.0, 46.0),
            true,
        )],
    }
}
