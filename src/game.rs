use macroquad::prelude::*;

use crate::data::{GameData, GameDataLoader};
use crate::engine::{day_engine, egg_engine, monster_engine, tower_engine, town_engine};
use crate::save::{SaveData, SaveRepository};
use crate::screens::{
    hatchery::{self, HatcheryAction},
    menu::{self, MenuAction},
    placeholder::{self, PlaceholderAction, PlaceholderKind},
    stable::{self, StableAction},
    tower::{self, TowerAction},
    town::{self, TownAction},
    AppScreen,
};
use crate::state::GameState;
use crate::ui;

pub struct Game {
    data: GameData,
    state: Option<GameState>,
    screen: AppScreen,
    status_message: String,
}

impl Game {
    pub async fn new() -> Self {
        let (data, status_message) = match GameDataLoader::load_embedded() {
            Ok(data) => (data, "Ready.".to_owned()),
            Err(error) => {
                eprintln!("Failed to load embedded data: {error}");
                (
                    GameData::fallback(),
                    format!("Loaded fallback data after content error: {error}"),
                )
            }
        };

        Self {
            data,
            state: None,
            screen: AppScreen::MainMenu,
            status_message,
        }
    }

    pub fn update(&mut self) {
        match self.screen {
            AppScreen::MainMenu => {
                let has_save = SaveRepository::exists();
                if let Some(action) = menu::handle_input(has_save) {
                    self.apply_menu_action(action);
                }
            }
            AppScreen::Town => {
                if let Some(state) = &self.state {
                    if let Some(action) = town::handle_input(state, &self.data) {
                        self.apply_town_action(action);
                    }
                } else {
                    self.screen = AppScreen::MainMenu;
                    self.status_message = "No active save. Start a new game.".to_owned();
                }
            }
            AppScreen::Hatchery => {
                if let Some(state) = &self.state {
                    if let Some(action) = hatchery::handle_input(state) {
                        self.apply_hatchery_action(action);
                    }
                } else {
                    self.screen = AppScreen::MainMenu;
                    self.status_message = "No active save. Start a new game.".to_owned();
                }
            }
            AppScreen::Stable => {
                if let Some(state) = &self.state {
                    if let Some(action) = stable::handle_input(state) {
                        self.apply_stable_action(action);
                    }
                } else {
                    self.screen = AppScreen::MainMenu;
                    self.status_message = "No active save. Start a new game.".to_owned();
                }
            }
            AppScreen::DungeonPrep => {
                if let Some(action) = placeholder::handle_input(PlaceholderKind::DungeonPrep) {
                    self.apply_placeholder_action(action);
                }
            }
            AppScreen::Tower => {
                if let Some(state) = &self.state {
                    if let Some(action) = tower::handle_input(state) {
                        self.apply_tower_action(action);
                    }
                } else {
                    self.screen = AppScreen::MainMenu;
                    self.status_message = "No active save. Start a new game.".to_owned();
                }
            }
            AppScreen::Combat => {
                if let Some(action) = placeholder::handle_input(PlaceholderKind::Combat) {
                    self.apply_placeholder_action(action);
                }
            }
            AppScreen::EndOfDay => {
                if let Some(action) = placeholder::handle_input(PlaceholderKind::EndOfDay) {
                    self.apply_placeholder_action(action);
                }
            }
        }
    }

    pub fn draw(&self) {
        clear_background(ui::BACKGROUND);
        set_camera(&ui::virtual_camera());

        match self.screen {
            AppScreen::MainMenu => {
                menu::draw(SaveRepository::exists(), &self.status_message);
            }
            AppScreen::Town => {
                if let Some(state) = &self.state {
                    town::draw(state, &self.data, &self.status_message);
                }
            }
            AppScreen::Hatchery => {
                if let Some(state) = &self.state {
                    hatchery::draw(state, &self.data, &self.status_message);
                }
            }
            AppScreen::Stable => {
                if let Some(state) = &self.state {
                    stable::draw(state, &self.data, &self.status_message);
                }
            }
            AppScreen::DungeonPrep => {
                placeholder::draw(PlaceholderKind::DungeonPrep, &self.status_message);
            }
            AppScreen::Tower => {
                if let Some(state) = &self.state {
                    tower::draw(state, &self.data, &self.status_message);
                }
            }
            AppScreen::Combat => {
                placeholder::draw(PlaceholderKind::Combat, &self.status_message);
            }
            AppScreen::EndOfDay => {
                placeholder::draw(PlaceholderKind::EndOfDay, &self.status_message);
            }
        }

        set_default_camera();
    }

    fn apply_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::NewGame => self.start_new_game(),
            MenuAction::LoadGame => self.load_game(),
        }
    }

    fn apply_town_action(&mut self, action: TownAction) {
        match action {
            TownAction::Sleep => {
                if let Some(state) = &mut self.state {
                    let result = day_engine::sleep(state);
                    self.status_message = result.summary;
                    self.screen = AppScreen::EndOfDay;
                }
            }
            TownAction::DungeonPrep => {
                self.screen = AppScreen::DungeonPrep;
                self.status_message = "Choose a party before entering the tower.".to_owned();
            }
            TownAction::OpenHatchery => self.open_facility("hatchery", AppScreen::Hatchery),
            TownAction::OpenStable => self.open_facility("stable", AppScreen::Stable),
            TownAction::Scavenge => {
                if let Some(state) = &mut self.state {
                    self.status_message = town_engine::scavenge_supplies(state).summary;
                }
            }
            TownAction::AdvanceBuilding(building_id) => {
                if let Some(state) = &mut self.state {
                    self.status_message =
                        town_engine::advance_building(state, &self.data, &building_id).summary;
                }
            }
            TownAction::Trade(trade) => {
                if let Some(state) = &mut self.state {
                    self.status_message = town_engine::trade_shop(state, &self.data, trade).summary;
                }
            }
            TownAction::GreetNpc(npc_id) => {
                if let Some(state) = &mut self.state {
                    self.status_message =
                        town_engine::greet_npc(state, &self.data, &npc_id).summary;
                }
            }
            TownAction::Save => self.save_game(),
            TownAction::Load => self.load_game(),
            TownAction::BackToMenu => {
                self.screen = AppScreen::MainMenu;
                self.status_message = "Returned to title.".to_owned();
            }
        }
    }

    fn apply_hatchery_action(&mut self, action: HatcheryAction) {
        match action {
            HatcheryAction::ToTown => {
                self.screen = AppScreen::Town;
                self.status_message = "Returned to tower camp.".to_owned();
            }
            HatcheryAction::DiscoverEgg => {
                if let Some(state) = &mut self.state {
                    self.status_message = egg_engine::discover_egg(state, &self.data).summary;
                }
            }
            HatcheryAction::WarmEgg(egg_id) => {
                if let Some(state) = &mut self.state {
                    self.status_message = egg_engine::warm_egg(state, &self.data, egg_id).summary;
                }
            }
            HatcheryAction::HatchEgg(egg_id) => {
                if let Some(state) = &mut self.state {
                    self.status_message = egg_engine::hatch_egg(state, &self.data, egg_id).summary;
                }
            }
        }
    }

    fn apply_stable_action(&mut self, action: StableAction) {
        match action {
            StableAction::ToTown => {
                self.screen = AppScreen::Town;
                self.status_message = "Returned to tower camp.".to_owned();
            }
            StableAction::ToggleParty(monster_id) => {
                if let Some(state) = &mut self.state {
                    self.status_message =
                        monster_engine::toggle_party_member(state, &self.data, monster_id).summary;
                }
            }
            StableAction::RemoveSlot(slot_index) => {
                if let Some(state) = &mut self.state {
                    self.status_message =
                        monster_engine::remove_party_slot(state, slot_index).summary;
                }
            }
        }
    }

    fn apply_placeholder_action(&mut self, action: PlaceholderAction) {
        match action {
            PlaceholderAction::ToTown => {
                self.screen = AppScreen::Town;
                self.status_message = "Returned to tower camp.".to_owned();
            }
            PlaceholderAction::ToTower => {
                self.enter_tower();
            }
            PlaceholderAction::ToCombat => {
                self.screen = AppScreen::Combat;
                self.status_message = "Combat rules are not implemented yet.".to_owned();
            }
        }
    }

    fn apply_tower_action(&mut self, action: TowerAction) {
        match action {
            TowerAction::Explore => {
                if let Some(state) = &mut self.state {
                    self.status_message = tower_engine::explore_room(state, &self.data).summary;
                }
            }
            TowerAction::ReturnToTown => {
                if let Some(state) = &mut self.state {
                    self.status_message = tower_engine::return_to_town(state, &self.data).summary;
                }
                self.screen = AppScreen::Town;
            }
            TowerAction::ToTown => {
                self.screen = AppScreen::Town;
                self.status_message = "Returned to tower camp.".to_owned();
            }
        }
    }

    fn open_facility(&mut self, building_id: &str, screen: AppScreen) {
        let Some(state) = &self.state else {
            self.status_message = "No active save. Start a new game.".to_owned();
            return;
        };

        if state.town.building_level(building_id) == 0 {
            let building_name = self
                .data
                .building(building_id)
                .map(|building| building.name.as_str())
                .unwrap_or(building_id);
            self.status_message = format!("Build the {building_name} first.");
            return;
        }

        self.screen = screen;
        self.status_message = "Facility opened.".to_owned();
    }

    fn enter_tower(&mut self) {
        let Some(state) = &mut self.state else {
            self.screen = AppScreen::MainMenu;
            self.status_message = "No active save. Start a new game.".to_owned();
            return;
        };

        let result = tower_engine::start_run(state, &self.data);
        let run_started = state.tower_run.is_some();
        self.status_message = result.summary;
        if run_started {
            self.screen = AppScreen::Tower;
        }
    }

    fn start_new_game(&mut self) {
        let state = GameState::new(&self.data);
        self.state = Some(state);
        self.screen = AppScreen::Town;
        self.status_message = "New save started beside the ruined tower.".to_owned();
    }

    fn save_game(&mut self) {
        let Some(state) = &self.state else {
            self.status_message = "Nothing to save yet.".to_owned();
            return;
        };

        let save_data = SaveData {
            version: self.data.config.save_version,
            state: state.clone(),
        };

        match SaveRepository::save(&save_data) {
            Ok(()) => {
                self.status_message = format!("Saved day {}.", state.day);
            }
            Err(error) => {
                self.status_message = format!("Save failed: {error}");
            }
        }
    }

    fn load_game(&mut self) {
        match SaveRepository::load() {
            Ok(save_data) => {
                if save_data.version > self.data.config.save_version {
                    self.status_message = format!(
                        "Save version {} is newer than supported version {}.",
                        save_data.version, self.data.config.save_version
                    );
                    return;
                }

                let loaded_day = save_data.state.day;
                self.state = Some(save_data.state);
                self.screen = AppScreen::Town;
                self.status_message = format!("Loaded save on day {loaded_day}.");
            }
            Err(error) => {
                self.status_message = format!("Load failed: {error}");
            }
        }
    }
}
