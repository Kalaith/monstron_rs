use super::*;
use crate::data::GameDataLoader;
use crate::engine::combat_engine::{player_action, start_encounter};
use crate::state::{CombatOutcome, GameState};

fn data_and_state() -> (crate::data::GameData, GameState) {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let state = GameState::new(&data);
    (data, state)
}

fn complete_with(
    data: &crate::data::GameData,
    state: &mut GameState,
    command: CombatCommand,
) -> CombatReplay {
    start_encounter(state, data, 1, false);
    for _ in 0..80 {
        if state
            .combat
            .as_ref()
            .is_some_and(|combat| combat.outcome.is_some())
        {
            break;
        }
        player_action(state, data, command);
    }
    CombatReplay::from_combat(
        state
            .combat
            .as_ref()
            .expect("combat should remain for replay"),
    )
}

#[test]
fn victory_replay_matches_every_command() {
    let (data, mut state) = data_and_state();
    let replay = complete_with(&data, &mut state, CombatCommand::Attack);
    assert_eq!(replay.expected_outcome, Some(CombatOutcome::Victory));
    assert_eq!(replay.run(&data), ReplayReport::Match);
}

#[test]
fn defeat_replay_matches_the_rescue_path() {
    let (data, mut state) = data_and_state();
    let starter = state.monster_roster.monster_mut(1).expect("starter exists");
    starter.hp = 1;
    starter.max_hp = 1;
    let replay = complete_with(&data, &mut state, CombatCommand::Defend);
    assert_eq!(replay.expected_outcome, Some(CombatOutcome::Defeat));
    assert_eq!(replay.run(&data), ReplayReport::Match);
}

#[test]
fn fleeing_replay_uses_the_recorded_seed() {
    let (data, mut state) = data_and_state();
    let replay = complete_with(&data, &mut state, CombatCommand::Flee);
    assert_eq!(replay.expected_outcome, Some(CombatOutcome::Fled));
    assert_eq!(replay.run(&data), ReplayReport::Match);
}

#[test]
fn item_use_replay_matches_healing_and_resource_consumption() {
    let (data, mut state) = data_and_state();
    state
        .monster_roster
        .monster_mut(1)
        .expect("starter exists")
        .hp = 2;
    start_encounter(&mut state, &data, 1, false);
    let herbs_before = state.resources.amount("herbs");
    player_action(&mut state, &data, CombatCommand::Item);
    let replay = CombatReplay::from_combat(state.combat.as_ref().expect("combat remains"));
    assert_eq!(state.resources.amount("herbs"), herbs_before - 1);
    assert_eq!(replay.run(&data), ReplayReport::Match);
}
