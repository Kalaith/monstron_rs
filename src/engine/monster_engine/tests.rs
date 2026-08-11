use super::*;
use crate::data::GameDataLoader;

#[test]
fn recovery_reduces_strain_and_heals_injury_timer() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    let monster = state.monster_roster.monster_mut(1).unwrap();
    monster.condition.fatigue = 5;
    monster.condition.injury_days = 1;
    monster.hp = 1;

    let result = recover_monsters(&mut state);
    let recovered = state.monster_roster.monster(1).unwrap();

    assert_eq!(result.fatigue_reduced, 1);
    assert_eq!(result.injuries_healed, 1);
    assert_eq!(result.rested, 1);
    assert_eq!(recovered.condition.fatigue, 3);
    assert_eq!(recovered.condition.injury_days, 0);
    assert_eq!(recovered.hp, recovered.max_hp);
}
