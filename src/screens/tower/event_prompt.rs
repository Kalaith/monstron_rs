use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

use super::TowerAction;
use crate::data::{GameData, TowerEventDefinition};
use crate::state::TowerRunState;
use crate::ui;

pub(super) fn handle_input(run: &TowerRunState) -> Option<TowerAction> {
    let pending = run.pending_event.as_ref()?;
    for (index, event_id) in pending.event_ids.iter().take(2).enumerate() {
        if ui::button_clicked(choice_rect(index), true) {
            return Some(TowerAction::ChooseEvent(event_id.clone()));
        }
    }
    if ui::button_clicked(leave_rect(), true) {
        return Some(TowerAction::LeaveEvent);
    }
    None
}

pub(super) fn draw(data: &GameData, run: &TowerRunState) {
    let Some(pending) = &run.pending_event else {
        return;
    };
    let location = data.tower_special_location(&pending.special_location_id);
    draw_rectangle(
        0.0,
        0.0,
        ui::VIEW_WIDTH,
        ui::VIEW_HEIGHT,
        Color::from_rgba(1, 5, 7, 190),
    );
    let panel = Rect::new(248.0, 98.0, 784.0, 512.0);
    super::draw_overlay_panel(panel);
    draw_ui_text_ex(
        location.map_or("Tower Landmark", |entry| entry.name.as_str()),
        panel.x + 34.0,
        panel.y + 48.0,
        TextParams {
            font_size: 32,
            color: super::gold_bright(),
            ..Default::default()
        },
    );
    super::draw_wrapped_line(
        location.map_or(
            "The party has found something the expedition journal cannot identify.",
            |entry| entry.description.as_str(),
        ),
        panel.x + 34.0,
        panel.y + 82.0,
        82,
        ui::TEXT_DIM,
    );

    for (index, event_id) in pending.event_ids.iter().take(2).enumerate() {
        draw_choice(data, data.tower_event(event_id), choice_rect(index), index);
    }
    ui::draw_button(leave_rect(), "LEAVE UNDISTURBED", true);
    draw_ui_text_ex(
        "Choose one approach. The expedition pauses until you tap a choice or LEAVE.",
        panel.x + 78.0,
        panel.y + panel.h - 24.0,
        TextParams {
            font_size: 14,
            color: ui::TEXT_DIM,
            ..Default::default()
        },
    );
}

fn draw_choice(data: &GameData, event: Option<&TowerEventDefinition>, rect: Rect, index: usize) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(13, 23, 24, 248),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, super::gold_dim());
    let label = event.map_or("Unknown Approach", |entry| entry.name.as_str());
    draw_ui_text_ex(
        &format!("{}  {}", index + 1, label),
        rect.x + 20.0,
        rect.y + 32.0,
        TextParams {
            font_size: 22,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    if let Some(event) = event {
        super::draw_wrapped_line(
            &event.narrative,
            rect.x + 20.0,
            rect.y + 56.0,
            78,
            ui::TEXT_DIM,
        );
        draw_ui_text_ex(
            &effect_summary(data, event),
            rect.x + 438.0,
            rect.y + 32.0,
            TextParams {
                font_size: 15,
                color: super::gold_bright(),
                ..Default::default()
            },
        );
    }
}

fn effect_summary(data: &GameData, event: &TowerEventDefinition) -> String {
    let mut effects = event
        .rewards
        .iter()
        .map(|reward| {
            format!(
                "+{} {}",
                reward.amount,
                data.resource_name(&reward.resource_id)
            )
        })
        .collect::<Vec<_>>();
    if event.party_healing > 0 {
        effects.push(format!("Heal {} each", event.party_healing));
    }
    if event.pressure_delta != 0 {
        effects.push(format!("Pressure {:+}", event.pressure_delta));
    }
    if !event.enemy_id.is_empty() {
        effects.push("Ambush".to_owned());
    }
    if effects.is_empty() {
        "Unknown consequence".to_owned()
    } else {
        effects.join("  ·  ")
    }
}

fn choice_rect(index: usize) -> Rect {
    Rect::new(282.0, 222.0 + index as f32 * 122.0, 716.0, 104.0)
}

fn leave_rect() -> Rect {
    Rect::new(525.0, 484.0, 230.0, 52.0)
}
