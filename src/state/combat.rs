use serde::{Deserialize, Serialize};

use crate::data::MonsterRole;
use crate::state::ResourceStack;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CombatState {
    pub floor: u32,
    pub round: u32,
    pub turn_index: usize,
    pub turn_order: Vec<CombatTurn>,
    pub allies: Vec<Combatant>,
    pub enemies: Vec<Combatant>,
    pub rewards: Vec<ResourceStack>,
    pub xp_reward: u32,
    pub log: Vec<String>,
    pub outcome: Option<CombatOutcome>,
    pub is_boss: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CombatTurn {
    pub side: CombatSide,
    pub slot: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CombatSide {
    Ally,
    Enemy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CombatOutcome {
    Victory,
    Defeat,
    Fled,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Combatant {
    pub name: String,
    pub source_id: String,
    pub monster_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<MonsterRole>,
    pub slot: usize,
    pub level: u32,
    pub max_hp: i32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub morale: i32,
    pub is_defending: bool,
    #[serde(default)]
    pub is_guarding: bool,
    #[serde(default)]
    pub is_marked: bool,
    pub visual_seed: u64,
}

impl CombatState {
    pub fn current_turn(&self) -> Option<CombatTurn> {
        self.turn_order.get(self.turn_index).copied()
    }

    pub fn is_player_turn(&self) -> bool {
        self.outcome.is_none()
            && self
                .current_turn()
                .is_some_and(|turn| turn.side == CombatSide::Ally)
    }

    pub fn add_log(&mut self, message: String) {
        self.log.push(message);
        if self.log.len() > 9 {
            let overflow = self.log.len() - 9;
            self.log.drain(0..overflow);
        }
    }
}

impl Combatant {
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}
