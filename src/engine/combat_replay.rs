use serde::{Deserialize, Serialize};

use crate::data::GameData;
use crate::engine::combat_engine::{self, CombatCommand};
use crate::state::{CombatOutcome, CombatReplayCommand, CombatState, CombatTurn, GameState};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CombatReplay {
    pub rng_seed: u64,
    pub roster: Vec<crate::state::Combatant>,
    pub encounter: ReplayEncounter,
    pub commands: Vec<ReplayCommand>,
    pub expected_outcome: Option<CombatOutcome>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReplayEncounter {
    pub floor: u32,
    pub is_boss: bool,
    pub enemies: Vec<crate::state::Combatant>,
    pub initial_turn_order: Vec<CombatTurn>,
    pub initial_round: u32,
    pub initial_turn_index: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReplayCommand {
    pub command: CombatReplayCommand,
    pub expected_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayReport {
    Match,
    Mismatch {
        command_index: usize,
        expected: String,
        actual: String,
    },
    OutcomeMismatch {
        expected: Option<CombatOutcome>,
        actual: Option<CombatOutcome>,
    },
}

impl CombatReplay {
    pub fn from_combat(combat: &CombatState) -> Self {
        let roster = if combat.replay_roster.is_empty() {
            combat.allies.clone()
        } else {
            combat.replay_roster.clone()
        };
        let enemies = if combat.replay_enemies.is_empty() {
            combat.enemies.clone()
        } else {
            combat.replay_enemies.clone()
        };
        Self {
            rng_seed: combat.rng_seed,
            roster,
            encounter: ReplayEncounter {
                floor: combat.floor,
                is_boss: combat.is_boss,
                enemies,
                initial_turn_order: if combat.replay_turn_order.is_empty() {
                    combat.turn_order.clone()
                } else {
                    combat.replay_turn_order.clone()
                },
                initial_round: combat.replay_round.max(1),
                initial_turn_index: combat.replay_turn_index,
            },
            commands: combat
                .command_history
                .iter()
                .map(|step| ReplayCommand {
                    command: step.command,
                    expected_digest: step.digest.clone(),
                })
                .collect(),
            expected_outcome: combat.outcome,
        }
    }

    pub fn run(&self, data: &GameData) -> ReplayReport {
        let mut state = GameState::new(data);
        state.combat = Some(CombatState {
            floor: self.encounter.floor,
            round: self.encounter.initial_round,
            turn_index: self.encounter.initial_turn_index,
            turn_order: self.encounter.initial_turn_order.clone(),
            allies: self.roster.clone(),
            enemies: self.encounter.enemies.clone(),
            rewards: Vec::new(),
            xp_reward: 0,
            log: Vec::new(),
            outcome: None,
            is_boss: self.encounter.is_boss,
            rng_seed: self.rng_seed,
            replay_roster: self.roster.clone(),
            replay_enemies: self.encounter.enemies.clone(),
            replay_turn_order: self.encounter.initial_turn_order.clone(),
            replay_round: self.encounter.initial_round,
            replay_turn_index: self.encounter.initial_turn_index,
            command_history: Vec::new(),
        });

        for (command_index, replay_command) in self.commands.iter().enumerate() {
            let command = match replay_command.command {
                CombatReplayCommand::Attack => CombatCommand::Attack,
                CombatReplayCommand::Skill => CombatCommand::Skill,
                CombatReplayCommand::Defend => CombatCommand::Defend,
                CombatReplayCommand::Item => CombatCommand::Item,
                CombatReplayCommand::Flee => CombatCommand::Flee,
            };
            combat_engine::player_action(&mut state, data, command);
            let actual = state
                .combat
                .as_ref()
                .map(combat_engine::combat_digest)
                .unwrap_or_else(|| "combat-ended".to_owned());
            if actual != replay_command.expected_digest {
                return ReplayReport::Mismatch {
                    command_index,
                    expected: replay_command.expected_digest.clone(),
                    actual,
                };
            }
        }

        let actual = state.combat.as_ref().and_then(|combat| combat.outcome);
        if actual != self.expected_outcome {
            return ReplayReport::OutcomeMismatch {
                expected: self.expected_outcome,
                actual,
            };
        }
        ReplayReport::Match
    }
}

#[cfg(test)]
mod tests;
