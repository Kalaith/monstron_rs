use super::*;
use crate::data::GameDataLoader;
use crate::engine::combat_engine::start_named_encounter;

#[test]
fn bulwark_spends_its_third_round_bracing() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_named_encounter(&mut state, &data, 5, false, Some("mirror_crab"));
    let combat = state.combat.as_mut().expect("combat should start");
    combat.round = 3;
    let ally_hp = combat.allies[0].hp;

    enemy_action(combat, 0);

    assert!(combat.enemies[0].is_defending);
    assert_eq!(combat.allies[0].hp, ally_hp);
    assert!(combat.log.iter().any(|line| line.contains("braces")));
}

#[test]
fn hexer_attack_drains_morale_and_speed() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_named_encounter(&mut state, &data, 6, false, Some("moonblind_owl"));
    let combat = state.combat.as_mut().expect("combat should start");
    combat.round = 2;
    let morale = combat.allies[0].morale;
    let speed = combat.allies[0].speed;

    enemy_action(combat, 0);

    assert_eq!(combat.allies[0].morale, morale - 7);
    assert_eq!(combat.allies[0].speed, (speed - 1).max(1));
    assert!(combat.log.iter().any(|line| line.contains("courage")));
}

#[test]
fn ambusher_opens_on_the_weakest_companion() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_named_encounter(&mut state, &data, 2, false, Some("archway_pouncer"));
    let combat = state.combat.as_mut().expect("combat should start");
    let mut weak_ally = combat.allies[0].clone();
    weak_ally.name = "Weak Test Ally".to_owned();
    weak_ally.slot = 1;
    weak_ally.hp = 6;
    weak_ally.max_hp = 20;
    combat.allies.push(weak_ally);
    combat.round = 1;
    let front_hp = combat.allies[0].hp;

    enemy_action(combat, 0);

    assert_eq!(combat.allies[0].hp, front_hp);
    assert!(combat.allies[1].hp < 6);
    assert!(combat
        .log
        .iter()
        .any(|line| line.contains("Weak Test Ally")));
}

#[test]
fn regenerator_mends_itself_before_attacking() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_named_encounter(&mut state, &data, 6, false, Some("rime_marrow"));
    let combat = state.combat.as_mut().expect("combat should start");
    combat.round = 2;
    combat.enemies[0].hp -= 12;
    let wounded_hp = combat.enemies[0].hp;

    enemy_action(combat, 0);

    assert!(combat.enemies[0].hp > wounded_hp);
    assert!(combat.log.iter().any(|line| line.contains("tower matter")));
}

#[test]
fn packleader_spends_its_second_round_rallying_the_pack() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_named_encounter(&mut state, &data, 2, false, Some("spore_herald"));
    let combat = state.combat.as_mut().expect("combat should start");
    combat.round = 2;
    let ally_hp = combat.allies[0].hp;
    let pack_attack = combat.enemies[1].attack;

    enemy_action(combat, 0);

    assert_eq!(combat.allies[0].hp, ally_hp);
    assert_eq!(combat.enemies[1].attack, pack_attack + 2);
    assert!(combat.log.iter().any(|line| line.contains("rallies")));
}

#[test]
fn sapper_breaks_defense_before_its_even_round_attack() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    start_named_encounter(&mut state, &data, 2, false, Some("mortar_termite"));
    let combat = state.combat.as_mut().expect("combat should start");
    combat.round = 2;
    combat.allies[0].is_defending = true;
    let defense = combat.allies[0].defense;

    enemy_action(combat, 0);

    assert!(!combat.allies[0].is_defending);
    assert_eq!(combat.allies[0].defense, (defense - 2).max(0));
    assert!(combat.log.iter().any(|line| line.contains("breaks")));
}
