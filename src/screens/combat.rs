use macroquad::prelude::*;

use crate::assets;
use crate::data::GameData;
use crate::engine::combat_engine::CombatCommand;
use crate::state::{CombatOutcome, CombatSide, CombatState, Combatant, GameState};
use crate::ui;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatAction {
    Command(CombatCommand),
    Continue,
}

pub fn handle_input(state: &GameState) -> Option<CombatAction> {
    let combat = state.combat.as_ref()?;

    if combat.outcome.is_some() {
        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || ui::button_clicked(continue_rect(), true)
        {
            return Some(CombatAction::Continue);
        }
        return None;
    }

    if !combat.is_player_turn() {
        return None;
    }

    if is_key_pressed(KeyCode::A) {
        return Some(CombatAction::Command(CombatCommand::Attack));
    }
    if is_key_pressed(KeyCode::S) {
        return Some(CombatAction::Command(CombatCommand::Skill));
    }
    if is_key_pressed(KeyCode::D) {
        return Some(CombatAction::Command(CombatCommand::Defend));
    }
    if is_key_pressed(KeyCode::I) {
        return Some(CombatAction::Command(CombatCommand::Item));
    }
    if is_key_pressed(KeyCode::F) || is_key_pressed(KeyCode::Escape) {
        return Some(CombatAction::Command(CombatCommand::Flee));
    }

    for (command, rect) in command_buttons() {
        if ui::button_clicked(rect, true) {
            return Some(CombatAction::Command(command));
        }
    }

    None
}

pub fn draw(state: &GameState, data: &GameData, status_message: &str) {
    draw_backdrop();
    if let Some(combat) = &state.combat {
        draw_header(combat);
        draw_formation(combat);
        draw_actions(combat);
        draw_rewards(combat, data);
        draw_log(combat);
    } else {
        draw_empty();
    }
    ui::draw_status(status_message);
}

fn draw_backdrop() {
    draw_rectangle(
        0.0,
        0.0,
        ui::VIEW_WIDTH,
        ui::VIEW_HEIGHT,
        Color::from_rgba(19, 21, 25, 255),
    );
    draw_rectangle(
        0.0,
        440.0,
        ui::VIEW_WIDTH,
        280.0,
        Color::from_rgba(35, 39, 42, 255),
    );
    draw_circle(980.0, 160.0, 130.0, Color::from_rgba(155, 92, 72, 26));
}

fn draw_header(combat: &CombatState) {
    ui::draw_panel(Rect::new(32.0, 24.0, ui::VIEW_WIDTH - 64.0, 78.0));
    draw_text_ex(
        &format!("Combat - Floor {}", combat.floor),
        58.0,
        72.0,
        TextParams {
            font_size: 34,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("Round {}", combat.round),
        520.0,
        70.0,
        TextParams {
            font_size: 24,
            color: ui::ACCENT,
            ..Default::default()
        },
    );
    let turn_text = if let Some(outcome) = combat.outcome {
        match outcome {
            CombatOutcome::Victory => "Victory",
            CombatOutcome::Defeat => "Defeat",
            CombatOutcome::Fled => "Fled",
        }
    } else if combat
        .current_turn()
        .is_some_and(|turn| turn.side == CombatSide::Ally)
    {
        "Party turn"
    } else {
        "Enemy turn"
    };
    draw_text_ex(
        turn_text,
        ui::VIEW_WIDTH - 230.0,
        70.0,
        TextParams {
            font_size: 24,
            color: ui::ACCENT,
            ..Default::default()
        },
    );
}

fn draw_formation(combat: &CombatState) {
    let rect = Rect::new(32.0, 124.0, 780.0, 330.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Formation", rect.x + 20.0, rect.y + 34.0);
    draw_text_ex(
        "Allies",
        rect.x + 58.0,
        rect.y + 78.0,
        TextParams {
            font_size: 22,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_text_ex(
        "Enemies",
        rect.x + 472.0,
        rect.y + 78.0,
        TextParams {
            font_size: 22,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );

    for combatant in &combat.allies {
        draw_combatant(combatant, true, ally_slot_rect(combatant.slot));
    }
    for combatant in &combat.enemies {
        draw_combatant(combatant, false, enemy_slot_rect(combatant.slot));
    }
}

fn draw_combatant(combatant: &Combatant, is_ally: bool, rect: Rect) {
    let fill = if combatant.is_alive() {
        Color::from_rgba(29, 38, 43, 230)
    } else {
        Color::from_rgba(29, 31, 33, 180)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, ui::PANEL_EDGE);

    if is_ally {
        assets::draw_monster_badge(combatant.visual_seed, rect.x + 10.0, rect.y + 16.0, 34.0);
    } else {
        draw_enemy_badge(combatant.visual_seed, rect.x + 10.0, rect.y + 18.0);
    }

    let name_color = if combatant.is_alive() {
        ui::TEXT_BRIGHT
    } else {
        ui::TEXT_DIM
    };
    draw_text_ex(
        &combatant.name,
        rect.x + 54.0,
        rect.y + 28.0,
        TextParams {
            font_size: 18,
            color: name_color,
            ..Default::default()
        },
    );
    draw_hp_bar(combatant, rect.x + 54.0, rect.y + 42.0, rect.w - 66.0);
    let row = if combatant.slot < 3 { "F" } else { "B" };
    draw_text_ex(
        &format!(
            "{}{} ATK {} DEF {}",
            row,
            combatant.slot + 1,
            combatant.attack,
            combatant.defense
        ),
        rect.x + 10.0,
        rect.y + rect.h - 22.0,
        TextParams {
            font_size: 12,
            color: ui::TEXT_DIM,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("SPD {} MOR {}", combatant.speed, combatant.morale),
        rect.x + 10.0,
        rect.y + rect.h - 8.0,
        TextParams {
            font_size: 12,
            color: ui::TEXT_DIM,
            ..Default::default()
        },
    );
}

fn draw_enemy_badge(seed: u64, x: f32, y: f32) {
    let r = 34.0 + (seed % 20) as f32;
    let color = Color::from_rgba(
        120 + (seed % 80) as u8,
        70 + ((seed >> 8) % 70) as u8,
        70 + ((seed >> 16) % 80) as u8,
        255,
    );
    draw_rectangle(x, y, 34.0, 34.0, color);
    draw_rectangle_lines(x, y, 34.0, 34.0, 2.0, ui::PANEL_EDGE);
    draw_circle(
        x + 17.0,
        y + 17.0,
        r.min(17.0),
        Color::from_rgba(10, 12, 14, 80),
    );
}

fn draw_hp_bar(combatant: &Combatant, x: f32, y: f32, width: f32) {
    draw_rectangle(x, y, width, 12.0, Color::from_rgba(20, 24, 28, 255));
    let ratio = combatant.hp.max(0) as f32 / combatant.max_hp.max(1) as f32;
    let color = if ratio < 0.3 {
        Color::from_rgba(210, 86, 72, 255)
    } else {
        ui::ACCENT
    };
    draw_rectangle(x, y, width * ratio.clamp(0.0, 1.0), 12.0, color);
    draw_rectangle_lines(x, y, width, 12.0, 1.0, ui::PANEL_EDGE);
    draw_text_ex(
        &format!("{}/{}", combatant.hp.max(0), combatant.max_hp),
        x,
        y + 28.0,
        TextParams {
            font_size: 13,
            color: ui::TEXT_DIM,
            ..Default::default()
        },
    );
}

fn draw_actions(combat: &CombatState) {
    let rect = Rect::new(32.0, 474.0, 780.0, 126.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Actions", rect.x + 20.0, rect.y + 32.0);

    if combat.outcome.is_some() {
        ui::draw_button(continue_rect(), "Continue", true);
        return;
    }

    let enabled = combat.is_player_turn();
    for (command, button_rect) in command_buttons() {
        let label = match command {
            CombatCommand::Attack => "Attack",
            CombatCommand::Skill => "Skill",
            CombatCommand::Defend => "Defend",
            CombatCommand::Item => "Herbs",
            CombatCommand::Flee => "Flee",
        };
        ui::draw_button(button_rect, label, enabled);
    }
}

fn draw_rewards(combat: &CombatState, data: &GameData) {
    let rect = Rect::new(836.0, 124.0, 412.0, 150.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Rewards", rect.x + 20.0, rect.y + 32.0);
    draw_text_ex(
        &format!("XP: {}", combat.xp_reward),
        rect.x + 20.0,
        rect.y + 68.0,
        TextParams {
            font_size: 20,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    let reward_text = combat
        .rewards
        .iter()
        .map(|reward| {
            format!(
                "{} {}",
                reward.amount,
                data.resource_name(&reward.resource_id)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    draw_text_ex(
        if reward_text.is_empty() {
            "Materials: none"
        } else {
            &reward_text
        },
        rect.x + 20.0,
        rect.y + 102.0,
        TextParams {
            font_size: 17,
            color: ui::TEXT_DIM,
            ..Default::default()
        },
    );
}

fn draw_log(combat: &CombatState) {
    let rect = Rect::new(836.0, 298.0, 412.0, 302.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Battle Log", rect.x + 20.0, rect.y + 32.0);
    for (index, message) in combat.log.iter().rev().take(6).enumerate() {
        draw_text_ex(
            message,
            rect.x + 20.0,
            rect.y + 68.0 + index as f32 * 34.0,
            TextParams {
                font_size: 16,
                color: ui::TEXT,
                ..Default::default()
            },
        );
    }
}

fn draw_empty() {
    let rect = Rect::new(250.0, 170.0, 780.0, 300.0);
    ui::draw_panel(rect);
    ui::draw_centered_text(
        "No Combat Active",
        ui::VIEW_WIDTH * 0.5,
        rect.y + 120.0,
        36,
        ui::TEXT_BRIGHT,
    );
}

fn ally_slot_rect(slot: usize) -> Rect {
    let column = slot % 3;
    let row = slot / 3;
    Rect::new(
        52.0 + column as f32 * 124.0,
        236.0 - row as f32 * 92.0,
        116.0,
        76.0,
    )
}

fn enemy_slot_rect(slot: usize) -> Rect {
    let column = slot % 3;
    let row = slot / 3;
    Rect::new(
        460.0 + column as f32 * 124.0,
        236.0 - row as f32 * 92.0,
        116.0,
        76.0,
    )
}

fn command_buttons() -> [(CombatCommand, Rect); 5] {
    [
        (CombatCommand::Attack, Rect::new(210.0, 538.0, 100.0, 36.0)),
        (CombatCommand::Skill, Rect::new(322.0, 538.0, 100.0, 36.0)),
        (CombatCommand::Defend, Rect::new(434.0, 538.0, 100.0, 36.0)),
        (CombatCommand::Item, Rect::new(546.0, 538.0, 100.0, 36.0)),
        (CombatCommand::Flee, Rect::new(658.0, 538.0, 100.0, 36.0)),
    ]
}

fn continue_rect() -> Rect {
    Rect::new(582.0, 538.0, 176.0, 36.0)
}
