use crate::data::GameData;
use crate::engine::combat_support::{
    add_boss_egg, advance_to_player_or_outcome, advance_turn, ally_attack, ally_skill, award_xp,
    build_allies, build_enemies, combined_rewards, defend, encounter_xp, flee_chance,
    flee_succeeds, rebuild_turn_order, record_floor_reached, reward_text, sync_allies,
    victory_rewards,
};
use crate::engine::{monster_engine, tower_engine};
use crate::state::{CombatOutcome, CombatSide, CombatState, GameState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatCommand {
    Attack,
    Skill,
    Defend,
    Item,
    Flee,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatDestination {
    Combat,
    Tower,
    Town,
}

pub struct CombatResult {
    pub summary: String,
}

pub struct CombatFinish {
    pub summary: String,
    pub destination: CombatDestination,
}

pub fn start_encounter(
    state: &mut GameState,
    data: &GameData,
    floor: u32,
    is_boss: bool,
) -> CombatResult {
    if state.combat.is_some() {
        return CombatResult {
            summary: "A combat encounter is already active.".to_owned(),
        };
    }

    let allies = build_allies(state);
    if allies.is_empty() {
        state.tower_run = None;
        return CombatResult {
            summary: "No battle-ready monsters remain. The party retreats to town.".to_owned(),
        };
    }

    let enemies = build_enemies(data, floor, is_boss);
    if enemies.is_empty() {
        return CombatResult {
            summary: format!("No enemy data is available for floor {floor}."),
        };
    }

    let mut combat = CombatState {
        floor,
        round: 1,
        turn_index: 0,
        turn_order: Vec::new(),
        allies,
        rewards: combined_rewards(data, &enemies),
        xp_reward: encounter_xp(data, &enemies),
        enemies,
        log: Vec::new(),
        outcome: None,
        is_boss,
    };
    combat.add_log(format!("Encounter started on floor {floor}."));
    rebuild_turn_order(&mut combat);
    advance_to_player_or_outcome(&mut combat);

    let summary = if is_boss {
        "Boss combat started.".to_owned()
    } else {
        "Enemy combat started.".to_owned()
    };
    state.combat = Some(combat);
    CombatResult { summary }
}

pub fn player_action(
    state: &mut GameState,
    data: &GameData,
    command: CombatCommand,
) -> CombatResult {
    if command == CombatCommand::Item {
        return use_item(state);
    }

    let Some(combat) = &mut state.combat else {
        return CombatResult {
            summary: "No combat encounter is active.".to_owned(),
        };
    };

    if combat.outcome.is_some() {
        return CombatResult {
            summary: "The encounter is already resolved.".to_owned(),
        };
    }

    let Some(turn) = combat.current_turn() else {
        return CombatResult {
            summary: "Combat turn order is empty.".to_owned(),
        };
    };
    if turn.side != CombatSide::Ally {
        advance_to_player_or_outcome(combat);
        return CombatResult {
            summary: "Enemies moved before the party could act.".to_owned(),
        };
    }

    let summary = match command {
        CombatCommand::Attack => ally_attack(combat, turn.slot),
        CombatCommand::Skill => ally_skill(combat, data, turn.slot),
        CombatCommand::Defend => defend(combat, turn.slot),
        CombatCommand::Flee => {
            let chance = flee_chance(combat);
            if flee_succeeds(combat) {
                combat.outcome = Some(CombatOutcome::Fled);
                combat.add_log(format!("The party breaks away from the fight ({chance}%)."));
                "The party flees toward town.".to_owned()
            } else {
                combat.add_log(format!("Escape failed despite a {chance}% chance."));
                "The party fails to find an escape route.".to_owned()
            }
        }
        CombatCommand::Item => unreachable!("item command is handled before borrowing combat"),
    };

    if combat.outcome.is_none() {
        advance_turn(combat);
        advance_to_player_or_outcome(combat);
    }

    CombatResult { summary }
}

pub fn finish_combat(state: &mut GameState, data: &GameData) -> CombatFinish {
    let Some(combat) = state.combat.take() else {
        return CombatFinish {
            summary: "No combat encounter is active.".to_owned(),
            destination: CombatDestination::Town,
        };
    };

    match combat.outcome {
        Some(CombatOutcome::Victory) => finish_victory(state, data, combat),
        Some(CombatOutcome::Defeat) => finish_defeat(state, combat),
        Some(CombatOutcome::Fled) => finish_flee(state, data, combat),
        None => {
            state.combat = Some(combat);
            CombatFinish {
                summary: "The fight is still underway.".to_owned(),
                destination: CombatDestination::Combat,
            }
        }
    }
}

fn use_item(state: &mut GameState) -> CombatResult {
    let Some(combat) = state.combat.as_ref() else {
        return CombatResult {
            summary: "No combat encounter is active.".to_owned(),
        };
    };
    let Some(turn) = combat.current_turn() else {
        return CombatResult {
            summary: "Combat turn order is empty.".to_owned(),
        };
    };
    if turn.side != CombatSide::Ally {
        return CombatResult {
            summary: "Items can only be used on an allied turn.".to_owned(),
        };
    }
    if state.resources.amount("herbs") <= 0 {
        return CombatResult {
            summary: "No herbs are available for a field dressing.".to_owned(),
        };
    }

    let _ = state.resources.spend(&[("herbs".to_owned(), 1)]);
    let combat = state.combat.as_mut().expect("combat was checked above");
    let heal = 10;
    let name = combat.allies[turn.slot].name.clone();
    let ally = &mut combat.allies[turn.slot];
    ally.hp = (ally.hp + heal).min(ally.max_hp);
    let summary = format!("{name} uses herbs and recovers {heal} HP.");
    combat.add_log(summary.clone());
    advance_turn(combat);
    advance_to_player_or_outcome(combat);

    CombatResult { summary }
}

fn finish_victory(state: &mut GameState, data: &GameData, combat: CombatState) -> CombatFinish {
    sync_allies(state, &combat);
    award_xp(state, combat.xp_reward);
    apply_victory_strain(state, &combat);
    record_floor_reached(state, data, combat.floor);
    let rewards = victory_rewards(&combat);

    if let Some(run) = &mut state.tower_run {
        for reward in &rewards {
            run.add_cargo(&reward.resource_id, reward.amount);
        }
        if combat.is_boss {
            add_boss_egg(run, data, combat.floor);
        }
        run.add_event(format!("Won combat on floor {}.", combat.floor));
    } else {
        for reward in &rewards {
            state.resources.add(&reward.resource_id, reward.amount);
        }
    }

    let summary = format!(
        "Victory on floor {}. Gained {} XP and {}. The party gains expedition strain.",
        combat.floor,
        combat.xp_reward,
        reward_text(data, &rewards)
    );
    state.activity_log.add(state.day, summary.clone());
    CombatFinish {
        summary,
        destination: CombatDestination::Tower,
    }
}

fn finish_defeat(state: &mut GameState, combat: CombatState) -> CombatFinish {
    for ally in &combat.allies {
        if let Some(monster_id) = ally.monster_id {
            if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
                monster.hp = 1;
                monster_engine::add_injury(monster, 2);
                monster_engine::add_fatigue(monster, 1);
            }
        }
    }
    state.tower_run = None;
    let summary = format!(
        "The party was defeated on floor {} and rescued back to town. Run cargo was lost, and the party needs recovery.",
        combat.floor
    );
    state.activity_log.add(state.day, summary.clone());
    CombatFinish {
        summary,
        destination: CombatDestination::Town,
    }
}

fn finish_flee(state: &mut GameState, data: &GameData, combat: CombatState) -> CombatFinish {
    sync_allies(state, &combat);
    apply_flee_strain(state, &combat);
    let summary = if state.tower_run.is_some() {
        let tower_summary = tower_engine::return_to_town(state, data).summary;
        format!("Fled combat. The party gains light strain. {tower_summary}")
    } else {
        "Fled combat and returned to town with light strain.".to_owned()
    };
    state.activity_log.add(state.day, summary.clone());
    CombatFinish {
        summary,
        destination: CombatDestination::Town,
    }
}

fn apply_victory_strain(state: &mut GameState, combat: &CombatState) {
    for ally in &combat.allies {
        let Some(monster_id) = ally.monster_id else {
            continue;
        };
        let Some(monster) = state.monster_roster.monster_mut(monster_id) else {
            continue;
        };
        if ally.hp <= 0 {
            monster_engine::add_injury(monster, 1);
        } else {
            monster_engine::add_fatigue(monster, 1);
        }
    }
}

fn apply_flee_strain(state: &mut GameState, combat: &CombatState) {
    for ally in &combat.allies {
        let Some(monster_id) = ally.monster_id else {
            continue;
        };
        let Some(monster) = state.monster_roster.monster_mut(monster_id) else {
            continue;
        };
        monster_engine::add_fatigue(monster, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameDataLoader;
    use crate::state::CombatTurn;

    #[test]
    fn victory_adds_fatigue_to_surviving_party_members() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);

        start_encounter(&mut state, &data, 1, false);
        state.combat.as_mut().unwrap().outcome = Some(CombatOutcome::Victory);
        finish_combat(&mut state, &data);

        let starter = state.monster_roster.monster(1).unwrap();
        assert_eq!(starter.condition.fatigue, 1);
        assert_eq!(starter.condition.injury_days, 0);
    }

    #[test]
    fn defeat_injures_and_tires_party_members() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);

        start_encounter(&mut state, &data, 1, false);
        state.combat.as_mut().unwrap().outcome = Some(CombatOutcome::Defeat);
        finish_combat(&mut state, &data);

        let starter = state.monster_roster.monster(1).unwrap();
        assert_eq!(starter.hp, 1);
        assert_eq!(starter.condition.injury_days, 2);
        assert_eq!(starter.condition.fatigue, 3);
    }

    #[test]
    fn scout_mark_adds_victory_loot_bonus() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);
        let starting_coins = state.resources.amount("coins");

        start_encounter(&mut state, &data, 1, false);
        let result = player_action(&mut state, &data, CombatCommand::Skill);
        assert!(result.summary.contains("marks"));
        assert!(state
            .combat
            .as_ref()
            .unwrap()
            .enemies
            .iter()
            .any(|enemy| enemy.is_marked));

        state.combat.as_mut().unwrap().outcome = Some(CombatOutcome::Victory);
        finish_combat(&mut state, &data);

        assert!(state.resources.amount("coins") >= starting_coins + 5);
    }

    #[test]
    fn tank_guard_redirects_back_row_pressure() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);
        let rootling = data.species("rootling").expect("rootling should exist");
        let rillfin = data.species("rillfin").expect("rillfin should exist");
        let tank_id = state
            .monster_roster
            .add_monster("Root".to_owned(), rootling, 0x7007);
        let back_id = state
            .monster_roster
            .add_monster("Ripple".to_owned(), rillfin, 0x7008);
        state.monster_roster.party_slots =
            vec![Some(tank_id), None, None, Some(back_id), None, None];

        start_encounter(&mut state, &data, 3, false);
        let tank_index = state
            .combat
            .as_ref()
            .unwrap()
            .allies
            .iter()
            .position(|ally| ally.monster_id == Some(tank_id))
            .unwrap();
        let enemy_index = 0;
        {
            let combat = state.combat.as_mut().unwrap();
            combat.floor = 3;
            combat.round = 1;
            combat.turn_order = vec![
                CombatTurn {
                    side: CombatSide::Ally,
                    slot: tank_index,
                },
                CombatTurn {
                    side: CombatSide::Enemy,
                    slot: enemy_index,
                },
            ];
            combat.turn_index = 0;
        }
        let back_hp_before = state
            .combat
            .as_ref()
            .unwrap()
            .allies
            .iter()
            .find(|ally| ally.monster_id == Some(back_id))
            .unwrap()
            .hp;

        let result = player_action(&mut state, &data, CombatCommand::Skill);
        assert!(result.summary.contains("guards"));

        let combat = state.combat.as_ref().unwrap();
        let tank = combat
            .allies
            .iter()
            .find(|ally| ally.monster_id == Some(tank_id))
            .unwrap();
        let back = combat
            .allies
            .iter()
            .find(|ally| ally.monster_id == Some(back_id))
            .unwrap();
        assert!(tank.hp < tank.max_hp);
        assert_eq!(back.hp, back_hp_before);
    }
}
