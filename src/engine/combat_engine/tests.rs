use super::*;
use crate::data::GameDataLoader;
use crate::state::{CombatTurn, TowerRunGoal};

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
    state.monster_roster.party_slots = vec![Some(tank_id), None, None, Some(back_id), None, None];

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
        combat.enemies[enemy_index].enemy_behavior = Some(crate::data::EnemyBehavior::Standard);
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
#[test]
fn named_tower_encounter_preserves_the_map_enemy_identity() {
    let data = crate::data::GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);

    start_named_encounter(&mut state, &data, 5, false, Some("glass_leech"));
    let combat = state.combat.as_ref().expect("combat should start");

    assert_eq!(combat.enemies.len(), 2);
    assert!(combat
        .enemies
        .iter()
        .all(|enemy| enemy.source_id == "glass_leech"));
}

#[test]
fn boss_victory_opens_the_live_crown_threshold() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    crate::engine::tower_engine::start_run(&mut state, &data, TowerRunGoal::PushDeeper);
    state.tower_run.as_mut().unwrap().current_floor = 10;
    start_named_encounter(&mut state, &data, 10, true, Some("verdant_crown"));
    state.combat.as_mut().unwrap().outcome = Some(CombatOutcome::Victory);

    finish_combat(&mut state, &data);

    let run = state
        .tower_run
        .as_ref()
        .expect("victory should return to tower");
    assert!(run.boss_defeated);
    assert!(run
        .event_log
        .iter()
        .any(|event| event.contains("threshold opens")));
    assert!(run
        .found_eggs
        .iter()
        .any(|egg| egg.egg_type_id == "boss_egg"));
}
