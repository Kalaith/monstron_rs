use serde::{Deserialize, Serialize};

use crate::state::GameState;

const GAME_NAME: &str = "monstron";
const SAVE_SLOT: &str = "slot_1";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SaveData {
    pub version: u32,
    pub state: GameState,
}

pub struct SaveRepository;

impl SaveRepository {
    pub fn save(save_data: &SaveData) -> Result<(), String> {
        macroquad_toolkit::persistence::save_to_slot_with_version(
            GAME_NAME,
            SAVE_SLOT,
            save_data,
            env!("CARGO_PKG_VERSION"),
        )
    }

    pub fn load() -> Result<SaveData, String> {
        macroquad_toolkit::persistence::load_from_slot(GAME_NAME, SAVE_SLOT)
    }

    pub fn exists() -> bool {
        macroquad_toolkit::persistence::slot_exists(GAME_NAME, SAVE_SLOT)
    }
}
