use crate::data::{Element, GameData, PassiveSkill, Temperament};
use crate::engine::{monster_engine, town_engine};
use crate::state::DailyCommitment;
use crate::state::{EggInheritance, GameState, MonsterArtProfile, MonsterInstance};

const GROVE_ID: &str = "breeding_grove";

pub struct BreedingResult {
    pub summary: String,
}

pub struct BreedingForecast {
    pub egg_options: Vec<(String, u32)>,
    pub species_options: Vec<String>,
    pub element_options: Vec<Element>,
    pub temperament_options: Vec<Temperament>,
    pub passive_options: Vec<PassiveSkill>,
    pub mutation_chance: u32,
    pub lineage_quality: u32,
}

pub fn breed_pair(
    state: &mut GameState,
    data: &GameData,
    first_id: u64,
    second_id: u64,
) -> BreedingResult {
    if state.town.building_level(GROVE_ID) == 0 {
        return BreedingResult {
            summary: "Build the Breeding Grove before pairing monsters.".to_owned(),
        };
    }
    if first_id == second_id {
        return BreedingResult {
            summary: "Choose two different monsters for a breeding pair.".to_owned(),
        };
    }
    if !town_engine::has_egg_capacity(state) {
        let current = state.egg_inventory.eggs.len();
        let capacity = town_engine::egg_capacity(state);
        return BreedingResult {
            summary: format!(
                "Hatchery egg capacity is full ({current}/{capacity}). Build or upgrade the Hatchery before breeding."
            ),
        };
    }

    let Some(first) = state.monster_roster.monster(first_id).cloned() else {
        return BreedingResult {
            summary: "The first parent is no longer in the roster.".to_owned(),
        };
    };
    let Some(second) = state.monster_roster.monster(second_id).cloned() else {
        return BreedingResult {
            summary: "The second parent is no longer in the roster.".to_owned(),
        };
    };

    if !pair_is_compatible(&first, &second) {
        return BreedingResult {
            summary: format!(
                "{} and {} need a shared element, shared role, or stronger bonds.",
                first.name, second.name
            ),
        };
    }

    if let Err(summary) = monster_engine::can_take_daily_action(state, &first) {
        return BreedingResult { summary };
    }
    if let Err(summary) = monster_engine::can_take_daily_action(state, &second) {
        return BreedingResult { summary };
    }

    let Some(egg_type) = select_egg_type(data, &first, &second, state.tower_progress.best_floor)
    else {
        return BreedingResult {
            summary: "No egg type can carry that lineage yet.".to_owned(),
        };
    };

    let cost = breeding_cost();
    if let Err(missing) = state.resources.spend(&cost) {
        return BreedingResult {
            summary: format!("Breeding needs {}.", cost_text(data, &missing)),
        };
    }

    let origin_floor = state.tower_progress.best_floor.max(1);
    let seed = breeding_seed(state, &first, &second);
    let inheritance = build_inheritance(data, &first, &second, egg_type, origin_floor, seed);
    let mutated = inheritance.mutated;
    let quality = inheritance.lineage_quality;
    let egg_id = state.egg_inventory.add_bred_egg(
        egg_type.id.clone(),
        egg_type.hatch_days,
        origin_floor,
        seed,
        inheritance,
    );
    raise_parent_bonds(state, data, first.id, second.id);
    commit_parent_recovery_time(state, first.id, second.id);

    let mutation_note = if mutated {
        " with a tower mutation"
    } else {
        ""
    };
    let summary = format!(
        "{} and {} nested {} #{}{} with {} lineage.",
        first.name,
        second.name,
        egg_type.name,
        egg_id,
        mutation_note,
        lineage_quality_label(quality)
    );
    state.activity_log.add(state.day, summary.clone());
    BreedingResult { summary }
}

pub fn forecast_pair(
    state: &GameState,
    data: &GameData,
    first: &MonsterInstance,
    second: &MonsterInstance,
) -> Option<BreedingForecast> {
    if !pair_is_compatible(first, second) {
        return None;
    }
    let origin_floor = state.tower_progress.best_floor.max(1);
    let egg_options = eligible_egg_types(data, first, second, origin_floor)
        .into_iter()
        .take(3)
        .enumerate()
        .map(|(index, egg_type)| {
            let weight = match index {
                0 => 60,
                1 => 30,
                _ => 10,
            };
            (egg_type.name.clone(), weight)
        })
        .collect::<Vec<_>>();
    let egg_type = select_egg_type(data, first, second, origin_floor)?;
    let seed = breeding_seed(state, first, second);
    let inheritance = build_inheritance(data, first, second, egg_type, origin_floor, seed);

    Some(BreedingForecast {
        egg_options,
        species_options: inheritance.species_options,
        element_options: inheritance.element_options,
        temperament_options: inheritance.temperament_options,
        passive_options: inheritance.passive_options,
        mutation_chance: mutation_chance(origin_floor),
        lineage_quality: lineage_quality(first, second, origin_floor),
    })
}

pub fn pair_is_compatible(first: &MonsterInstance, second: &MonsterInstance) -> bool {
    first.element == second.element || first.role == second.role || first.bond + second.bond >= 6
}

pub fn compatibility_label(first: &MonsterInstance, second: &MonsterInstance) -> String {
    if first.element == second.element {
        format!("Shared {} element", first.element)
    } else if first.role == second.role {
        format!("Shared {} role", first.role)
    } else if first.bond + second.bond >= 6 {
        "Bonded pair".to_owned()
    } else {
        "Needs shared trait or bond 6+".to_owned()
    }
}

pub fn mutation_chance(origin_floor: u32) -> u32 {
    (origin_floor.max(1) * 4).min(35)
}

pub fn lineage_quality(
    first: &MonsterInstance,
    second: &MonsterInstance,
    origin_floor: u32,
) -> u32 {
    let mut quality = 1;
    if first.element == second.element {
        quality += 1;
    }
    if first.role == second.role {
        quality += 1;
    }
    if first.bond + second.bond >= 6 {
        quality += 1;
    }
    if first.bond + second.bond >= 10 || origin_floor >= 6 {
        quality += 1;
    }
    quality.min(5)
}

pub fn lineage_quality_label(quality: u32) -> &'static str {
    match quality {
        0 | 1 => "plain",
        2 => "steady",
        3 => "resonant",
        4 => "potent",
        _ => "mythic",
    }
}

pub fn breeding_cost() -> Vec<(String, i32)> {
    vec![("herbs".to_owned(), 2)]
}

fn select_egg_type<'a>(
    data: &'a GameData,
    first: &MonsterInstance,
    second: &MonsterInstance,
    best_floor: u32,
) -> Option<&'a crate::data::EggTypeDefinition> {
    let eligible = eligible_egg_types(data, first, second, best_floor);

    let seed = first.visual_seed ^ second.visual_seed ^ best_floor as u64;
    eligible
        .get((seed as usize) % eligible.len().max(1))
        .copied()
}

fn eligible_egg_types<'a>(
    data: &'a GameData,
    first: &MonsterInstance,
    second: &MonsterInstance,
    best_floor: u32,
) -> Vec<&'a crate::data::EggTypeDefinition> {
    let eligible = data
        .egg_types
        .iter()
        .filter(|egg_type| egg_type.discovery_floor <= best_floor.max(1))
        .filter(|egg_type| {
            egg_type.possible_species.contains(&first.species_id)
                || egg_type.possible_species.contains(&second.species_id)
        })
        .collect::<Vec<_>>();
    if eligible.is_empty() {
        data.egg_types.iter().take(1).collect()
    } else {
        eligible
    }
}

fn build_inheritance(
    data: &GameData,
    first: &MonsterInstance,
    second: &MonsterInstance,
    egg_type: &crate::data::EggTypeDefinition,
    origin_floor: u32,
    seed: u64,
) -> EggInheritance {
    let mutated = seed % 100 < mutation_chance(origin_floor) as u64;
    let mut inheritance = EggInheritance {
        parent_ids: vec![first.id, second.id],
        species_options: unique_strings(vec![first.species_id.clone(), second.species_id.clone()]),
        element_options: unique_values(vec![first.element, second.element]),
        temperament_options: unique_values(vec![first.temperament, second.temperament]),
        passive_options: unique_values(vec![first.passive, second.passive]),
        mutation_floor: origin_floor,
        mutated,
        lineage_quality: lineage_quality(first, second, origin_floor),
        art_profile: MonsterArtProfile::default(),
    };

    if mutated {
        prepend_string(
            &mut inheritance.species_options,
            mutation_species(data, egg_type, seed),
        );
        prepend_value(&mut inheritance.element_options, mutation_element(seed));
        prepend_value(
            &mut inheritance.temperament_options,
            mutation_temperament(seed),
        );
        prepend_value(&mut inheritance.passive_options, mutation_passive(seed));
    }

    inheritance.art_profile = lineage_art_profile(data, &inheritance, egg_type, seed);
    inheritance
}

fn lineage_art_profile(
    data: &GameData,
    inheritance: &EggInheritance,
    egg_type: &crate::data::EggTypeDefinition,
    seed: u64,
) -> MonsterArtProfile {
    let species_id = inheritance
        .species_options
        .first()
        .or_else(|| egg_type.possible_species.first());
    let Some(species_id) = species_id else {
        return MonsterArtProfile::default();
    };
    let Some(species) = data.species(species_id) else {
        return MonsterArtProfile::default();
    };

    MonsterArtProfile::from_traits(
        species,
        inheritance
            .element_options
            .first()
            .copied()
            .unwrap_or(species.element),
        inheritance
            .temperament_options
            .first()
            .copied()
            .unwrap_or(species.temperament),
        inheritance
            .passive_options
            .first()
            .copied()
            .unwrap_or(species.passive),
        species.town_skill,
        seed,
    )
}

fn breeding_seed(state: &GameState, first: &MonsterInstance, second: &MonsterInstance) -> u64 {
    let parent_mix = first.visual_seed.rotate_left(9) ^ second.visual_seed.rotate_right(7);
    parent_mix ^ state.day as u64 * 97 ^ state.egg_inventory.next_id * 0x9E37
}

fn raise_parent_bonds(state: &mut GameState, data: &GameData, first_id: u64, second_id: u64) {
    let mut starter_reached = false;
    for monster in &mut state.monster_roster.monsters {
        if monster.id == first_id || monster.id == second_id {
            monster.bond += 1;
            if monster.name == data.config.starter_name && monster.bond >= 4 {
                starter_reached = true;
            }
        }
    }

    if starter_reached && state.story_flags.add("starter_slime_breeding_bond") {
        state.activity_log.add(
            state.day,
            format!(
                "{} nudges the new egg closer to the grove brazier.",
                data.config.starter_name
            ),
        );
    }
}

fn commit_parent_recovery_time(state: &mut GameState, first_id: u64, second_id: u64) {
    for monster_id in [first_id, second_id] {
        if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
            monster.condition.commitment = DailyCommitment::Breeding;
            monster_engine::add_fatigue(monster, 1);
        }
    }
}

fn mutation_species(
    data: &GameData,
    egg_type: &crate::data::EggTypeDefinition,
    seed: u64,
) -> String {
    if let Some(species_id) = egg_type
        .possible_species
        .get((seed.rotate_left(5) as usize) % egg_type.possible_species.len().max(1))
    {
        return species_id.clone();
    }
    data.monster_species
        .get((seed as usize) % data.monster_species.len().max(1))
        .map(|species| species.id.clone())
        .unwrap_or_else(|| "slime".to_owned())
}

fn mutation_element(seed: u64) -> Element {
    [Element::Water, Element::Fire, Element::Earth][(seed.rotate_left(3) as usize) % 3]
}

fn mutation_temperament(seed: u64) -> Temperament {
    [
        Temperament::Loyal,
        Temperament::Patient,
        Temperament::Curious,
        Temperament::Brave,
        Temperament::Restless,
        Temperament::Gentle,
    ][(seed.rotate_left(11) as usize) % 6]
}

fn mutation_passive(seed: u64) -> PassiveSkill {
    [
        PassiveSkill::FindsSmallLoot,
        PassiveSkill::ResistsPoison,
        PassiveSkill::DetectsEggs,
        PassiveSkill::FindsStone,
        PassiveSkill::BurnsBrambles,
        PassiveSkill::SoothesInjuries,
    ][(seed.rotate_left(17) as usize) % 6]
}

fn unique_strings(values: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}

fn unique_values<T: Copy + Eq>(values: Vec<T>) -> Vec<T> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}

fn prepend_string(values: &mut Vec<String>, value: String) {
    values.retain(|existing| existing != &value);
    values.insert(0, value);
}

fn prepend_value<T: Copy + Eq>(values: &mut Vec<T>, value: T) {
    values.retain(|existing| existing != &value);
    values.insert(0, value);
}

fn cost_text(data: &GameData, cost: &[(String, i32)]) -> String {
    cost.iter()
        .map(|(resource_id, amount)| format!("{} {}", amount, data.resource_name(resource_id)))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameDataLoader;
    use crate::engine::egg_engine;
    use crate::state::GameState;

    #[test]
    fn breeding_creates_an_inherited_hatchable_egg() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);
        state.town.set_building_level("breeding_grove", 1);
        state.town.set_building_level("hatchery", 1);
        state.resources.add("herbs", 10);
        state.tower_progress.best_floor = 5;

        let rillfin = data.species("rillfin").expect("rillfin should exist");
        let second_id = state
            .monster_roster
            .add_monster("Ripple".to_owned(), rillfin, 0xBEE5_7001);
        let first_id = 1;
        let first_bond = state.monster_roster.monster(first_id).unwrap().bond;
        let second_bond = state.monster_roster.monster(second_id).unwrap().bond;

        let result = breed_pair(&mut state, &data, first_id, second_id);
        assert!(result.summary.contains("nested"));
        assert_eq!(state.egg_inventory.eggs.len(), 1);
        assert_eq!(
            state.monster_roster.monster(first_id).unwrap().bond,
            first_bond + 1
        );
        assert_eq!(
            state.monster_roster.monster(second_id).unwrap().bond,
            second_bond + 1
        );

        let egg_id = state.egg_inventory.eggs[0].id;
        let inheritance = state.egg_inventory.eggs[0]
            .inheritance
            .clone()
            .expect("bred egg should carry inheritance");
        assert!(inheritance.parent_ids.contains(&first_id));
        assert!(inheritance.parent_ids.contains(&second_id));
        assert!(inheritance.species_options.iter().any(|id| id == "slime"));
        assert!(inheritance.species_options.iter().any(|id| id == "rillfin"));
        assert!(inheritance.lineage_quality >= 2);
        assert!(!inheritance.art_profile.species_hint.is_empty());

        state.egg_inventory.egg_mut(egg_id).unwrap().days_remaining = 0;
        let hatch = egg_engine::hatch_egg(&mut state, &data, egg_id);
        assert!(hatch.summary.contains("hatched"));
        let child = state.monster_roster.monsters.last().unwrap();
        assert!(inheritance.species_options.contains(&child.species_id));
        assert!(inheritance.element_options.contains(&child.element));
        assert!(inheritance.temperament_options.contains(&child.temperament));
        assert!(inheritance.passive_options.contains(&child.passive));
        assert!(child.bond >= inheritance.lineage_quality);
        assert!(!child.art_profile.palette.is_empty());
    }

    #[test]
    fn breeding_respects_hatchery_egg_capacity_before_spending_costs() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);
        state.town.set_building_level("breeding_grove", 1);
        state.town.set_building_level("hatchery", 1);
        state.resources.add("herbs", 10);

        for seed in 0..3 {
            state
                .egg_inventory
                .add_egg("mossy_egg".to_owned(), 1, 1, 0x300 + seed);
        }
        let rillfin = data.species("rillfin").expect("rillfin should exist");
        let second_id = state
            .monster_roster
            .add_monster("Ripple".to_owned(), rillfin, 0xBEE5_7001);
        let herbs_before = state.resources.amount("herbs");

        let result = breed_pair(&mut state, &data, 1, second_id);

        assert!(result.summary.contains("Hatchery egg capacity is full"));
        assert_eq!(state.resources.amount("herbs"), herbs_before);
        assert_eq!(state.egg_inventory.eggs.len(), 3);
    }
}
