use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

use super::{draw_overlay_panel, gold_bright};
use crate::data::GameData;
use crate::engine::tower_engine;
use crate::state::TowerRunState;
use crate::ui;

pub(super) fn draw(data: &GameData, run: &TowerRunState) {
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
    if let Some((progress, contract)) = tower_engine::contract_progress(run, data) {
        let marker = if run.contract_complete {
            " COMPLETE"
        } else {
            ""
        };
        draw_ui_text_ex(
            &format!(
                "{}  {}/{}{}",
                contract.name,
                progress.min(contract.target_amount),
                contract.target_amount,
                marker
            ),
            panel.x + 16.0,
            panel.y + 49.0,
            TextParams {
                font_size: 12,
                color: if run.contract_complete {
                    gold_bright()
                } else {
                    ui::TEXT_DIM
                },
                ..Default::default()
            },
        );
    }
    if !run.blessings.is_empty() {
        let labels = run
            .blessings
            .iter()
            .map(|blessing| blessing.label())
            .collect::<Vec<_>>()
            .join(" · ");
        draw_ui_text_ex(
            &format!("Blessings: {labels}"),
            panel.x + 16.0,
            panel.y + 69.0,
            TextParams {
                font_size: 11,
                color: gold_bright(),
                ..Default::default()
            },
        );
    }
    for (index, message) in run.event_log.iter().rev().take(2).enumerate() {
        draw_ui_text_ex(
            &format!("• {}", message),
            panel.x + 16.0,
            panel.y + 88.0 + index as f32 * 22.0,
            TextParams {
                font_size: 12,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    }
}
