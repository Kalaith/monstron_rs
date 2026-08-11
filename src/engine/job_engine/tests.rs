use super::*;
use crate::data::GameDataLoader;
use crate::state::GameState;

#[test]
fn daily_jobs_produce_resources_and_bond() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut state = GameState::new(&data);
    state.town.set_building_level("workshop", 1);
    let monster_id = 1;
    let starting_herbs = state.resources.amount("herbs");
    let starting_bond = state.monster_roster.monster(monster_id).unwrap().bond;

    assign_job(&mut state, &data, monster_id, TownJobKind::Forage);
    let result = run_daily_jobs(&mut state, &data);

    assert!(result.summary.contains("Town jobs produced"));
    assert!(state.resources.amount("herbs") > starting_herbs);
    assert_eq!(
        state.monster_roster.monster(monster_id).unwrap().bond,
        starting_bond + 1
    );
}
