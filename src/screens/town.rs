use macroquad::prelude::*;

use crate::assets;
use crate::data::GameData;
use crate::engine::town_engine::{self, ShopTrade};
use crate::state::GameState;
use crate::ui;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TownAction {
    Sleep,
    DungeonPrep,
    OpenHatchery,
    OpenStable,
    Scavenge,
    AdvanceBuilding(String),
    Trade(ShopTrade),
    GreetNpc(String),
    Save,
    Load,
    BackToMenu,
}

pub fn handle_input(state: &GameState, data: &GameData) -> Option<TownAction> {
    if is_key_pressed(KeyCode::Space) {
        return Some(TownAction::Sleep);
    }
    if is_key_pressed(KeyCode::S) {
        return Some(TownAction::Save);
    }
    if is_key_pressed(KeyCode::D) {
        return Some(TownAction::DungeonPrep);
    }
    if is_key_pressed(KeyCode::H) {
        return Some(TownAction::OpenHatchery);
    }
    if is_key_pressed(KeyCode::R) {
        return Some(TownAction::OpenStable);
    }
    if is_key_pressed(KeyCode::C) {
        return Some(TownAction::Scavenge);
    }
    if is_key_pressed(KeyCode::L) {
        return Some(TownAction::Load);
    }
    if is_key_pressed(KeyCode::Escape) {
        return Some(TownAction::BackToMenu);
    }

    for (index, building) in data.buildings.iter().enumerate() {
        let current_level = state.town.building_level(&building.id);
        if current_level < building.max_level
            && ui::button_clicked(building_button_rect(index), true)
        {
            return Some(TownAction::AdvanceBuilding(building.id.clone()));
        }
    }

    for npc in &data.npcs {
        if ui::button_clicked(npc_button_rect(&npc.id), true) {
            return Some(TownAction::GreetNpc(npc.id.clone()));
        }
    }

    let shop_unlocked = state.town.building_level("shop") > 0;
    for (trade, rect) in shop_buttons() {
        if ui::button_clicked(rect, shop_unlocked) {
            return Some(TownAction::Trade(trade));
        }
    }

    for (action, rect) in action_buttons() {
        if ui::button_clicked(rect, true) {
            return Some(action);
        }
    }

    None
}

pub fn draw(state: &GameState, data: &GameData, status_message: &str) {
    draw_town_backdrop();
    draw_header(state);
    draw_resources(state, data);
    draw_buildings(state, data);
    draw_roster(state, data);
    draw_npcs(state, data);
    draw_tower_progress(state);
    draw_log(state);
    draw_shop(state);
    draw_actions();
    ui::draw_status(status_message);
}

fn draw_town_backdrop() {
    draw_rectangle(
        0.0,
        0.0,
        ui::VIEW_WIDTH,
        ui::VIEW_HEIGHT,
        Color::from_rgba(24, 29, 34, 255),
    );
    draw_circle(1050.0, 130.0, 90.0, Color::from_rgba(218, 178, 92, 35));
    draw_rectangle(
        0.0,
        430.0,
        ui::VIEW_WIDTH,
        ui::VIEW_HEIGHT - 430.0,
        Color::from_rgba(37, 55, 45, 255),
    );

    for index in 0..9 {
        let x = 72.0 + index as f32 * 145.0;
        let h = 50.0 + (index % 3) as f32 * 24.0;
        draw_rectangle(x, 430.0 - h, 90.0, h, Color::from_rgba(56, 62, 70, 255));
        draw_triangle(
            vec2(x - 8.0, 430.0 - h),
            vec2(x + 45.0, 390.0 - h),
            vec2(x + 98.0, 430.0 - h),
            Color::from_rgba(74, 76, 82, 255),
        );
    }

    draw_rectangle(
        580.0,
        150.0,
        120.0,
        280.0,
        Color::from_rgba(88, 99, 112, 255),
    );
    draw_triangle(
        vec2(640.0, 50.0),
        vec2(562.0, 150.0),
        vec2(718.0, 150.0),
        Color::from_rgba(66, 75, 88, 255),
    );
}

fn draw_header(state: &GameState) {
    ui::draw_panel(Rect::new(24.0, 20.0, ui::VIEW_WIDTH - 48.0, 72.0));
    draw_text_ex(
        "Tower Camp",
        48.0,
        66.0,
        TextParams {
            font_size: 34,
            color: ui::TEXT_BRIGHT,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("Town Rank {}", town_engine::town_rank(state)),
        262.0,
        64.0,
        TextParams {
            font_size: 24,
            color: ui::ACCENT,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("Day {}", state.day),
        ui::VIEW_WIDTH - 170.0,
        66.0,
        TextParams {
            font_size: 28,
            color: ui::ACCENT,
            ..Default::default()
        },
    );
}

fn draw_resources(state: &GameState, data: &GameData) {
    let rect = Rect::new(24.0, 112.0, 220.0, 212.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Resources", rect.x + 16.0, rect.y + 32.0);

    for (index, resource) in data.resources.iter().enumerate() {
        let y = rect.y + 62.0 + index as f32 * 24.0;
        draw_text_ex(
            &resource.name,
            rect.x + 16.0,
            y,
            TextParams {
                font_size: 18,
                color: ui::TEXT,
                ..Default::default()
            },
        );
        let amount = state.resources.amount(&resource.id);
        let label = amount.to_string();
        draw_text_ex(
            &label,
            rect.x + rect.w - 20.0 - measure_text(&label, None, 18, 1.0).width,
            y,
            TextParams {
                font_size: 18,
                color: ui::TEXT_BRIGHT,
                ..Default::default()
            },
        );
    }
}

fn draw_buildings(state: &GameState, data: &GameData) {
    let rect = Rect::new(260.0, 112.0, 520.0, 360.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Buildings", rect.x + 18.0, rect.y + 32.0);

    for (index, building) in data.buildings.iter().enumerate() {
        let current_level = state.town.building_level(&building.id);
        let y = rect.y + 64.0 + index as f32 * 56.0;
        let status = if current_level == 0 {
            "Plan ready".to_owned()
        } else {
            format!("Level {}/{}", current_level, building.max_level)
        };
        let cost = town_engine::next_cost(data, &building.id, current_level);
        let cost_text = if current_level >= building.max_level {
            "Fully upgraded".to_owned()
        } else {
            format!("Cost: {}", town_engine::cost_text(data, &cost))
        };

        draw_text_ex(
            &building.name,
            rect.x + 18.0,
            y,
            TextParams {
                font_size: 21,
                color: ui::TEXT_BRIGHT,
                ..Default::default()
            },
        );
        draw_text_ex(
            &status,
            rect.x + 196.0,
            y,
            TextParams {
                font_size: 18,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
        draw_text_ex(
            &cost_text,
            rect.x + 18.0,
            y + 23.0,
            TextParams {
                font_size: 15,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );

        let label = if current_level == 0 {
            "Build"
        } else if current_level >= building.max_level {
            "Max"
        } else {
            "Upgrade"
        };
        ui::draw_button(
            building_button_rect(index),
            label,
            current_level < building.max_level,
        );
    }
}

fn draw_roster(state: &GameState, data: &GameData) {
    let rect = Rect::new(800.0, 112.0, ui::VIEW_WIDTH - 824.0, 180.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Monster Roster", rect.x + 18.0, rect.y + 32.0);

    if state.monster_roster.monsters.is_empty() {
        draw_text_ex(
            "No monsters have joined yet.",
            rect.x + 18.0,
            rect.y + 82.0,
            TextParams {
                font_size: 20,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
        return;
    }

    for (index, monster) in state.monster_roster.monsters.iter().take(2).enumerate() {
        let y = rect.y + 60.0 + index as f32 * 50.0;
        assets::draw_monster_badge(monster.visual_seed, rect.x + 18.0, y - 24.0, 34.0);
        let species_name = data
            .species(&monster.species_id)
            .map(|species| species.name.as_str())
            .unwrap_or(monster.species_id.as_str());
        draw_text_ex(
            &format!("{} the {}", monster.name, species_name),
            rect.x + 64.0,
            y,
            TextParams {
                font_size: 21,
                color: ui::TEXT_BRIGHT,
                ..Default::default()
            },
        );
        draw_text_ex(
            &format!(
                "Lv {}  HP {}/{}  Bond {}  {}",
                monster.level, monster.hp, monster.max_hp, monster.bond, monster.role
            ),
            rect.x + 64.0,
            y + 23.0,
            TextParams {
                font_size: 16,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
    }
}

fn draw_npcs(state: &GameState, data: &GameData) {
    let rect = Rect::new(800.0, 312.0, ui::VIEW_WIDTH - 824.0, 160.0);
    ui::draw_panel(rect);
    ui::draw_section_title("NPC Services", rect.x + 18.0, rect.y + 32.0);

    for (index, npc) in data.npcs.iter().enumerate() {
        let y = rect.y + 62.0 + index as f32 * 32.0;
        draw_text_ex(
            &format!("{} - {}", npc.name, npc.service),
            rect.x + 18.0,
            y,
            TextParams {
                font_size: 17,
                color: ui::TEXT_BRIGHT,
                ..Default::default()
            },
        );
        draw_text_ex(
            &format!("Friendship {}", state.npc_friendship(&npc.id)),
            rect.x + 254.0,
            y,
            TextParams {
                font_size: 16,
                color: ui::TEXT_DIM,
                ..Default::default()
            },
        );
        ui::draw_button(npc_button_rect(&npc.id), "Greet", true);
    }
}

fn draw_tower_progress(state: &GameState) {
    let rect = Rect::new(24.0, 344.0, 220.0, 128.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Tower", rect.x + 16.0, rect.y + 32.0);
    draw_text_ex(
        &format!("Best floor: {}", state.tower_progress.best_floor),
        rect.x + 16.0,
        rect.y + 70.0,
        TextParams {
            font_size: 18,
            color: ui::TEXT,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("Unlocked floor: {}", state.tower_progress.unlocked_floor),
        rect.x + 16.0,
        rect.y + 100.0,
        TextParams {
            font_size: 18,
            color: ui::TEXT,
            ..Default::default()
        },
    );
}

fn draw_log(state: &GameState) {
    let rect = Rect::new(260.0, 492.0, 520.0, 132.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Activity Log", rect.x + 18.0, rect.y + 30.0);

    let visible = state.activity_log.entries.iter().rev().take(3);
    for (index, entry) in visible.enumerate() {
        let y = rect.y + 60.0 + index as f32 * 24.0;
        draw_text_ex(
            &format!("Day {}: {}", entry.day, entry.message),
            rect.x + 18.0,
            y,
            TextParams {
                font_size: 16,
                color: ui::TEXT,
                ..Default::default()
            },
        );
    }
}

fn draw_shop(state: &GameState) {
    let rect = Rect::new(800.0, 492.0, ui::VIEW_WIDTH - 824.0, 132.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Shop", rect.x + 18.0, rect.y + 30.0);

    let shop_unlocked = state.town.building_level("shop") > 0;
    let helper = if shop_unlocked {
        "Buy basics or sell surplus herbs."
    } else {
        "Build the shop to unlock trades."
    };
    draw_text_ex(
        helper,
        rect.x + 18.0,
        rect.y + 58.0,
        TextParams {
            font_size: 15,
            color: ui::TEXT_DIM,
            ..Default::default()
        },
    );

    for (trade, button_rect) in shop_buttons() {
        let label = match trade {
            ShopTrade::BuyHerbs => "Buy Herbs",
            ShopTrade::BuyStone => "Buy Stone",
            ShopTrade::SellHerbs => "Sell Herbs",
        };
        ui::draw_button(button_rect, label, shop_unlocked);
    }
}

fn draw_actions() {
    let rect = Rect::new(24.0, 492.0, 220.0, 132.0);
    ui::draw_panel(rect);
    ui::draw_section_title("Camp Actions", rect.x + 16.0, rect.y + 30.0);

    for (action, button_rect) in action_buttons() {
        let label = match action {
            TownAction::Scavenge => "Scavenge",
            TownAction::Sleep => "Sleep",
            TownAction::OpenHatchery => "Hatchery",
            TownAction::OpenStable => "Stable",
            TownAction::DungeonPrep => "Tower",
            TownAction::Save => "Save",
            TownAction::Load => "Load",
            TownAction::BackToMenu => "Title",
            TownAction::AdvanceBuilding(_) | TownAction::Trade(_) | TownAction::GreetNpc(_) => "",
        };
        ui::draw_button(button_rect, label, true);
    }
}

fn building_button_rect(index: usize) -> Rect {
    Rect::new(672.0, 148.0 + index as f32 * 56.0, 88.0, 34.0)
}

fn npc_button_rect(npc_id: &str) -> Rect {
    let index = match npc_id {
        "mara" => 0,
        "bram" => 1,
        "lio" => 2,
        _ => 0,
    };
    Rect::new(1154.0, 357.0 + index as f32 * 32.0, 76.0, 26.0)
}

fn shop_buttons() -> [(ShopTrade, Rect); 3] {
    [
        (ShopTrade::BuyHerbs, Rect::new(982.0, 556.0, 86.0, 30.0)),
        (ShopTrade::BuyStone, Rect::new(1076.0, 556.0, 86.0, 30.0)),
        (ShopTrade::SellHerbs, Rect::new(1170.0, 556.0, 86.0, 30.0)),
    ]
}

fn action_buttons() -> Vec<(TownAction, Rect)> {
    vec![
        (TownAction::Scavenge, Rect::new(42.0, 532.0, 88.0, 22.0)),
        (TownAction::Sleep, Rect::new(138.0, 532.0, 88.0, 22.0)),
        (TownAction::OpenHatchery, Rect::new(42.0, 558.0, 88.0, 22.0)),
        (TownAction::OpenStable, Rect::new(138.0, 558.0, 88.0, 22.0)),
        (TownAction::DungeonPrep, Rect::new(42.0, 584.0, 88.0, 22.0)),
        (TownAction::Save, Rect::new(138.0, 584.0, 88.0, 22.0)),
        (TownAction::Load, Rect::new(42.0, 610.0, 88.0, 22.0)),
        (TownAction::BackToMenu, Rect::new(138.0, 610.0, 88.0, 22.0)),
    ]
}
