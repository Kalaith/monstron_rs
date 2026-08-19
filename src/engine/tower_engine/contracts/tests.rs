use super::*;
use crate::data::GameDataLoader;
use crate::state::{TowerFoundEgg, TowerRunGoal};

#[test]
fn egg_contract_awards_its_bonus_exactly_once() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut run = TowerRunState::new(1, 9, TowerRunGoal::EggHunt);
    for seed in [1, 2] {
        run.found_eggs.push(TowerFoundEgg {
            egg_type_id: "mossy_egg".to_owned(),
            hatch_days: 1,
            origin_floor: 1,
            palette_seed: seed,
        });
    }

    let completion = refresh_contract(&mut run, &data);
    let cargo_after_completion = run.cargo_amount();
    let repeated = refresh_contract(&mut run, &data);

    assert!(completion.unwrap().contains("Nest Survey"));
    assert!(run.contract_complete);
    assert_eq!(cargo_after_completion, 6);
    assert_eq!(run.cargo_amount(), cargo_after_completion);
    assert!(repeated.is_none());
}

#[test]
fn older_run_without_contract_id_uses_its_goal_contract() {
    let data = GameDataLoader::load_embedded().expect("embedded data should load");
    let mut run = TowerRunState::new(1, 9, TowerRunGoal::Salvage);
    run.contract_id.clear();
    run.add_cargo("wood", 9);

    let (progress, contract) = contract_progress(&run, &data).expect("contract should resolve");

    assert_eq!(progress, 9);
    assert_eq!(contract.id, "salvage");
}
