use macroquad::prelude::*;

use crate::data::{GameData, GameDataLoader};
use crate::engine::{
    combat_engine::{self, CombatDestination},
    tower_engine, town_engine,
};
use crate::save::{SaveData, SaveRepository};
use crate::screens::{
    breeding,
    combat::{self, CombatAction},
    hatchery,
    menu::{self, MenuAction, SettingsAction},
    placeholder::{self, PlaceholderAction, PlaceholderKind},
    shop, stable,
    tower::{self, TowerAction},
    town::{self, TownAction},
    workshop, AppScreen,
};
use crate::state::TowerRunGoal;
use crate::state::{GameState, TownJobKind};
use crate::ui;

pub struct Game {
    pub(crate) data: GameData,
    pub(crate) state: Option<GameState>,
    pub(crate) screen: AppScreen,
    pub(crate) status_message: String,
    pub(crate) town_menu_open: bool,
    tower_guide_open: bool,
    tower_guide_page: usize,
    title_texture: Texture2D,
    fullscreen_enabled: bool,
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

        let title_texture =
            Texture2D::from_file_with_format(include_bytes!("../hatchspire_title.png"), None);
        title_texture.set_filter(FilterMode::Linear);

        Self {
            data,
            state: None,
            screen: AppScreen::MainMenu,
            status_message,
            town_menu_open: false,
            tower_guide_open: false,
            tower_guide_page: 0,
            title_texture,
            fullscreen_enabled: false,
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
            AppScreen::Settings => {
                if let Some(action) = menu::handle_settings_input() {
                    self.apply_settings_action(action);
                }
            }
            AppScreen::Town => {
                if let Some(state) = &self.state {
                    if let Some(action) = town::handle_input(state, &self.data, self.town_menu_open)
                    {
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
            AppScreen::Breeding => {
                if let Some(state) = &self.state {
                    if let Some(action) = breeding::handle_input(state) {
                        self.apply_breeding_action(action);
                    }
                } else {
                    self.screen = AppScreen::MainMenu;
                    self.status_message = "No active save. Start a new game.".to_owned();
                }
            }
            AppScreen::Workshop => {
                if let Some(state) = &self.state {
                    if let Some(action) = workshop::handle_input(state) {
                        self.apply_workshop_action(action);
                    }
                } else {
                    self.screen = AppScreen::MainMenu;
                    self.status_message = "No active save. Start a new game.".to_owned();
                }
            }
            AppScreen::Shop => {
                if let Some(action) = shop::handle_input() {
                    self.apply_shop_action(action);
                }
            }
            AppScreen::DungeonPrep => {
                if let Some(action) = placeholder::handle_input(PlaceholderKind::DungeonPrep) {
                    self.apply_placeholder_action(action);
                }
            }
            AppScreen::Tower => {
                if let Some(state) = &self.state {
                    if let Some(action) = tower::handle_input(
                        state,
                        &self.data,
                        self.tower_guide_open,
                        self.tower_guide_page,
                    ) {
                        self.apply_tower_action(action);
                    }
                } else {
                    self.screen = AppScreen::MainMenu;
                    self.status_message = "No active save. Start a new game.".to_owned();
                }
            }
            AppScreen::Combat => {
                if let Some(state) = &self.state {
                    if let Some(action) = combat::handle_input(state) {
                        self.apply_combat_action(action);
                    }
                } else {
                    self.screen = AppScreen::MainMenu;
                    self.status_message = "No active save. Start a new game.".to_owned();
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
                menu::draw(&self.title_texture, SaveRepository::exists());
            }
            AppScreen::Settings => {
                menu::draw_settings(self.fullscreen_enabled);
            }
            AppScreen::Town => {
                if let Some(state) = &self.state {
                    town::draw(state, &self.data, &self.status_message, self.town_menu_open);
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
            AppScreen::Breeding => {
                if let Some(state) = &self.state {
                    breeding::draw(state, &self.data, &self.status_message);
                }
            }
            AppScreen::Workshop => {
                if let Some(state) = &self.state {
                    workshop::draw(state, &self.data, &self.status_message);
                }
            }
            AppScreen::Shop => {
                if let Some(state) = &self.state {
                    shop::draw(state, &self.data, &self.status_message);
                }
            }
            AppScreen::DungeonPrep => {
                placeholder::draw(PlaceholderKind::DungeonPrep, &self.status_message);
            }
            AppScreen::Tower => {
                if let Some(state) = &self.state {
                    tower::draw(
                        state,
                        &self.data,
                        &self.status_message,
                        self.tower_guide_open,
                        self.tower_guide_page,
                    );
                }
            }
            AppScreen::Combat => {
                if let Some(state) = &self.state {
                    combat::draw(state, &self.data, &self.status_message);
                }
            }
            AppScreen::EndOfDay => {
                placeholder::draw(PlaceholderKind::EndOfDay, &self.status_message);
            }
        }

        set_default_camera();
    }

    /// Seed a specific scene for the screenshot harness. Bypasses normal
    /// facility-unlock gating so a fresh save can still reach these screens.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "town" => self.begin_capture_fixture(AppScreen::Town),
            "hatchery" => self.begin_capture_fixture(AppScreen::Hatchery),
            "stable" => self.begin_capture_fixture(AppScreen::Stable),
            "breeding" => self.begin_capture_fixture(AppScreen::Breeding),
            "workshop" => self.begin_capture_fixture(AppScreen::Workshop),
            "shop" => self.begin_capture_fixture(AppScreen::Shop),
            "tower" => {
                self.begin_capture_fixture(AppScreen::Town);
                self.enter_tower(TowerRunGoal::Balanced);
            }
            "combat" => {
                self.begin_capture_fixture(AppScreen::Town);
                if let Some(state) = &mut self.state {
                    let result = combat_engine::start_encounter(state, &self.data, 1, false);
                    self.status_message = result.summary;
                    self.screen = AppScreen::Combat;
                }
            }
            "mainmenu" => {
                self.state = None;
                self.screen = AppScreen::MainMenu;
                self.status_message = "Ready.".to_owned();
            }
            _ => {
                // Default: boot state is the main menu.
            }
        }
    }

    fn begin_capture_fixture(&mut self, screen: AppScreen) {
        self.start_new_game();
        let Some(state) = &mut self.state else {
            return;
        };
        for building_id in ["hatchery", "stable", "breeding_grove", "workshop", "shop"] {
            state.town.set_building_level(building_id, 1);
        }
        for (resource_id, amount) in [
            ("coins", 80),
            ("wood", 60),
            ("stone", 50),
            ("ore", 8),
            ("herbs", 30),
            ("crystal", 4),
        ] {
            state.resources.add(resource_id, amount);
        }
        if let Some(species) = self.data.species("rillfin") {
            state
                .monster_roster
                .add_monster("Ripple".to_owned(), species, 0xBEE5_7001);
        }
        if let Some(species) = self.data.species("emberkit") {
            state
                .monster_roster
                .add_monster("Ember".to_owned(), species, 0xF17E_2002);
        }
        let _ = state.monster_roster.assign_to_party(2);
        let _ = state.monster_roster.assign_to_party(3);
        if screen == AppScreen::Stable {
            if let Some(monster) = state.monster_roster.monster_mut(2) {
                monster.condition.fatigue = 3;
            }
            if let Some(monster) = state.monster_roster.monster_mut(3) {
                monster.condition.injury_days = 1;
            }
        }
        if screen == AppScreen::Workshop {
            state.town.set_monster_job(1, TownJobKind::Forage);
        }
        state
            .egg_inventory
            .add_egg("mossy_egg".to_owned(), 0, 3, 0xA7C4_0001);
        self.screen = screen;
        self.town_menu_open = false;
        self.status_message =
            "Seeded verification scene. Tap a visible control to continue.".to_owned();
    }

    fn apply_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::NewGame => self.start_new_game(),
            MenuAction::LoadGame => self.load_game(),
            MenuAction::Settings => {
                self.screen = AppScreen::Settings;
            }
            MenuAction::ExitGame => {
                macroquad::miniquad::window::quit();
            }
        }
    }

    fn apply_settings_action(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::ToggleFullscreen => {
                self.fullscreen_enabled = !self.fullscreen_enabled;
                set_fullscreen(self.fullscreen_enabled);
            }
            SettingsAction::Back => {
                self.screen = AppScreen::MainMenu;
            }
        }
    }

    fn apply_town_action(&mut self, action: TownAction) {
        match action {
            TownAction::Sleep => {
                self.town_menu_open = false;
                if let Some(state) = &mut self.state {
                    let result = town_engine::reduce(state, &self.data, &TownAction::Sleep);
                    self.status_message = result.summary;
                    self.screen = AppScreen::EndOfDay;
                }
            }
            TownAction::DungeonPrep => {
                self.town_menu_open = false;
                self.screen = AppScreen::DungeonPrep;
                self.status_message = "Choose a party before entering the tower.".to_owned();
            }
            TownAction::OpenMenu => {
                self.town_menu_open = true;
            }
            TownAction::CloseMenu => {
                self.town_menu_open = false;
            }
            TownAction::OpenHatchery => self.open_facility("hatchery", AppScreen::Hatchery),
            TownAction::OpenStable => self.open_facility("stable", AppScreen::Stable),
            TownAction::OpenBreeding => {
                self.open_facility("breeding_grove", AppScreen::Breeding);
            }
            TownAction::OpenWorkshop => self.open_facility("workshop", AppScreen::Workshop),
            TownAction::OpenShop => self.open_facility("shop", AppScreen::Shop),
            TownAction::Scavenge => {
                self.town_menu_open = false;
                if let Some(state) = &mut self.state {
                    self.status_message =
                        town_engine::reduce(state, &self.data, &TownAction::Scavenge).summary;
                }
            }
            TownAction::AdvanceBuilding(building_id) => {
                self.town_menu_open = false;
                if let Some(state) = &mut self.state {
                    self.status_message = town_engine::reduce(
                        state,
                        &self.data,
                        &TownAction::AdvanceBuilding(building_id),
                    )
                    .summary;
                }
            }
            TownAction::Trade(trade) => {
                self.town_menu_open = false;
                if let Some(state) = &mut self.state {
                    self.status_message =
                        town_engine::reduce(state, &self.data, &TownAction::Trade(trade)).summary;
                }
            }
            TownAction::GreetNpc(npc_id) => {
                self.town_menu_open = false;
                if let Some(state) = &mut self.state {
                    self.status_message =
                        town_engine::reduce(state, &self.data, &TownAction::GreetNpc(npc_id))
                            .summary;
                }
            }
            TownAction::Save => self.save_game(),
            TownAction::Load => self.load_game(),
            TownAction::BackToMenu => {
                self.town_menu_open = false;
                self.screen = AppScreen::MainMenu;
                self.status_message = "Returned to title.".to_owned();
            }
        }
    }

    fn apply_placeholder_action(&mut self, action: PlaceholderAction) {
        match action {
            PlaceholderAction::ToTown => {
                self.screen = AppScreen::Town;
                self.status_message = "Returned to tower camp.".to_owned();
            }
            PlaceholderAction::ToTower(goal) => {
                self.enter_tower(goal);
            }
        }
    }

    fn apply_tower_action(&mut self, action: TowerAction) {
        match action {
            TowerAction::Move(dx, dy) => {
                if let Some(state) = &mut self.state {
                    let result = tower_engine::move_party(state, &self.data, dx, dy);
                    let returned_to_town = result.returned_to_town;
                    self.status_message = result.summary;
                    if returned_to_town {
                        self.screen = AppScreen::Town;
                    }
                    if let Some(encounter) = result.encounter {
                        let combat_result = combat_engine::start_named_encounter(
                            state,
                            &self.data,
                            encounter.floor,
                            encounter.is_boss,
                            encounter.enemy_id.as_deref(),
                        );
                        self.status_message = combat_result.summary;
                        if state.combat.is_some() {
                            self.screen = AppScreen::Combat;
                        }
                    }
                }
            }
            TowerAction::RouteTo(x, y) => {
                if let Some(state) = &mut self.state {
                    let result = tower_engine::route_party_to(state, &self.data, (x, y));
                    let returned_to_town = result.returned_to_town;
                    self.status_message = result.summary;
                    if returned_to_town {
                        self.screen = AppScreen::Town;
                    }
                    if let Some(encounter) = result.encounter {
                        let combat_result = combat_engine::start_named_encounter(
                            state,
                            &self.data,
                            encounter.floor,
                            encounter.is_boss,
                            encounter.enemy_id.as_deref(),
                        );
                        self.status_message = combat_result.summary;
                        if state.combat.is_some() {
                            self.screen = AppScreen::Combat;
                        }
                    }
                }
            }
            TowerAction::Explore => {
                if let Some(state) = &mut self.state {
                    let result = tower_engine::explore_party(state, &self.data);
                    let returned_to_town = result.returned_to_town;
                    self.status_message = result.summary;
                    if returned_to_town {
                        self.screen = AppScreen::Town;
                    }
                    if let Some(encounter) = result.encounter {
                        let combat_result = combat_engine::start_named_encounter(
                            state,
                            &self.data,
                            encounter.floor,
                            encounter.is_boss,
                            encounter.enemy_id.as_deref(),
                        );
                        self.status_message = combat_result.summary;
                        if state.combat.is_some() {
                            self.screen = AppScreen::Combat;
                        }
                    }
                }
            }
            TowerAction::Survey => {
                if let Some(state) = &mut self.state {
                    self.status_message = tower_engine::survey_floor(state, &self.data).summary;
                }
            }
            TowerAction::Camp => {
                if let Some(state) = &mut self.state {
                    self.status_message = tower_engine::camp_party(state, &self.data).summary;
                }
            }
            TowerAction::ChooseEvent(event_id) => {
                if let Some(state) = &mut self.state {
                    let result = tower_engine::choose_special_event(state, &self.data, &event_id);
                    self.status_message = result.summary;
                    if let Some(encounter) = result.encounter {
                        let combat_result = combat_engine::start_named_encounter(
                            state,
                            &self.data,
                            encounter.floor,
                            encounter.is_boss,
                            encounter.enemy_id.as_deref(),
                        );
                        self.status_message = combat_result.summary;
                        if state.combat.is_some() {
                            self.screen = AppScreen::Combat;
                        }
                    }
                }
            }
            TowerAction::LeaveEvent => {
                if let Some(state) = &mut self.state {
                    self.status_message =
                        tower_engine::leave_special_event(state, &self.data).summary;
                }
            }
            TowerAction::ReturnToTown => {
                if let Some(state) = &mut self.state {
                    self.status_message = tower_engine::return_to_town(state, &self.data).summary;
                }
                self.screen = AppScreen::Town;
                self.tower_guide_open = false;
            }
            TowerAction::ToTown => {
                self.screen = AppScreen::Town;
                self.tower_guide_open = false;
                self.status_message = "Returned to tower camp.".to_owned();
            }
            TowerAction::OpenGuide => {
                self.tower_guide_open = true;
                self.tower_guide_page = 0;
            }
            TowerAction::CloseGuide => self.tower_guide_open = false,
            TowerAction::GuidePage(delta) => {
                self.tower_guide_page = if delta < 0 {
                    self.tower_guide_page
                        .saturating_sub(delta.unsigned_abs() as usize)
                } else {
                    self.tower_guide_page.saturating_add(delta as usize)
                };
            }
        }
    }

    fn apply_combat_action(&mut self, action: CombatAction) {
        match action {
            CombatAction::Command(command) => {
                if let Some(state) = &mut self.state {
                    self.status_message =
                        combat_engine::reduce_command(state, &self.data, command).summary;
                }
            }
            CombatAction::Continue => {
                if let Some(state) = &mut self.state {
                    let finish = combat_engine::finish_combat(state, &self.data);
                    self.status_message = finish.summary;
                    self.screen = match finish.destination {
                        CombatDestination::Combat => AppScreen::Combat,
                        CombatDestination::Tower => AppScreen::Tower,
                        CombatDestination::Town => AppScreen::Town,
                    };
                }
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
            self.status_message =
                format!("Build the {building_name} first. Tap its Upgrade button on the town map.");
            return;
        }

        self.screen = screen;
        self.town_menu_open = false;
        self.status_message = "Facility opened.".to_owned();
    }

    fn enter_tower(&mut self, goal: TowerRunGoal) {
        let Some(state) = &mut self.state else {
            self.screen = AppScreen::MainMenu;
            self.status_message = "No active save. Start a new game.".to_owned();
            return;
        };

        let result = tower_engine::start_run(state, &self.data, goal);
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
        self.town_menu_open = false;
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
                let mut loaded_state = save_data.state;
                loaded_state.monster_roster.ensure_art_profiles(&self.data);
                tower_engine::ensure_map(&mut loaded_state, &self.data);
                self.state = Some(loaded_state);
                self.screen = AppScreen::Town;
                self.town_menu_open = false;
                self.status_message = format!("Loaded save on day {loaded_day}.");
            }
            Err(error) => {
                self.status_message = format!("Load failed: {error}");
            }
        }
    }
}
