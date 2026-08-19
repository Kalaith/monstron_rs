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
