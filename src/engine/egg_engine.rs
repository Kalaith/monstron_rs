use crate::data::{GameData, MonsterRole};
use crate::state::GameState;

const HATCHERY_ID: &str = "hatchery";

pub struct EggResult {
    pub summary: String,
}

pub fn discover_egg(state: &mut GameState, data: &GameData) -> EggResult {
    if state.town.building_level(HATCHERY_ID) == 0 {
        return EggResult {
            summary: "Build the hatchery before keeping tower eggs.".to_owned(),
        };
    }

    let eligible: Vec<_> = data
        .egg_types
        .iter()
        .filter(|egg| egg.discovery_floor <= state.tower_progress.unlocked_floor.max(1))
        .collect();

    let Some(egg_type) = eligible
        .get(((state.day as u64 + state.egg_inventory.next_id) as usize) % eligible.len().max(1))
        .copied()
    else {
        return EggResult {
            summary: "No egg definitions are available.".to_owned(),
        };
    };

    let seed = 0xE66_0000 + state.day as u64 * 97 + state.egg_inventory.next_id * 31;
    let id = state.egg_inventory.add_egg(
        egg_type.id.clone(),
        egg_type.hatch_days,
        egg_type.discovery_floor,
        seed,
    );
    let summary = format!(
        "Recovered {} #{} from the tower edge. It will hatch in {} day(s).",
        egg_type.name, id, egg_type.hatch_days
    );
    state.activity_log.add(state.day, summary.clone());

    EggResult { summary }
}

pub fn warm_egg(state: &mut GameState, data: &GameData, egg_id: u64) -> EggResult {
    if state.town.building_level(HATCHERY_ID) == 0 {
        return EggResult {
            summary: "Build the hatchery before warming eggs.".to_owned(),
        };
    }

    let Some(egg) = state.egg_inventory.eggs.iter().find(|egg| egg.id == egg_id) else {
        return EggResult {
            summary: "That egg is no longer in the hatchery.".to_owned(),
        };
    };

    if egg.days_remaining == 0 {
        return EggResult {
            summary: "That egg is already ready to hatch.".to_owned(),
        };
    }

    let cost = vec![("herbs".to_owned(), 1)];
    if let Err(missing) = state.resources.spend(&cost) {
        return EggResult {
            summary: format!("Warming needs {}.", cost_text(data, &missing)),
        };
    }

    let Some(egg) = state.egg_inventory.egg_mut(egg_id) else {
        return EggResult {
            summary: "That egg is no longer in the hatchery.".to_owned(),
        };
    };

    egg.days_remaining = egg.days_remaining.saturating_sub(1);
    let egg_name = data
        .egg_type(&egg.egg_type_id)
        .map(|egg_type| egg_type.name.as_str())
        .unwrap_or(egg.egg_type_id.as_str());
    let summary = format!(
        "Warmed {} #{}. {} day(s) remain.",
        egg_name, egg.id, egg.days_remaining
    );
    state.activity_log.add(state.day, summary.clone());

    EggResult { summary }
}

pub fn hatch_egg(state: &mut GameState, data: &GameData, egg_id: u64) -> EggResult {
    if state.town.building_level(HATCHERY_ID) == 0 {
        return EggResult {
            summary: "Build the hatchery before hatching eggs.".to_owned(),
        };
    }

    let Some(existing) = state.egg_inventory.eggs.iter().find(|egg| egg.id == egg_id) else {
        return EggResult {
            summary: "That egg is no longer in the hatchery.".to_owned(),
        };
    };

    if existing.days_remaining > 0 {
        return EggResult {
            summary: format!("That egg needs {} more day(s).", existing.days_remaining),
        };
    }

    let Some(egg) = state.egg_inventory.remove_egg(egg_id) else {
        return EggResult {
            summary: "That egg is no longer in the hatchery.".to_owned(),
        };
    };
    let Some(egg_type) = data.egg_type(&egg.egg_type_id) else {
        return EggResult {
            summary: format!("Missing egg definition '{}'.", egg.egg_type_id),
        };
    };

    let inherited = egg.inheritance.as_ref();
    let species_pool = match inherited {
        Some(inheritance) if !inheritance.species_options.is_empty() => {
            inheritance.species_options.as_slice()
        }
        _ => egg_type.possible_species.as_slice(),
    };
    let mutation_active = inherited.is_some_and(|inheritance| inheritance.mutated);
    let species_seed = if mutation_active { 0 } else { egg.palette_seed };
    let Some(species_id) = select_species_id(species_seed, species_pool) else {
        return EggResult {
            summary: format!("{} has no possible species.", egg_type.name),
        };
    };
    let Some(species) = data.species(species_id) else {
        return EggResult {
            summary: format!("Missing species definition '{species_id}'."),
        };
    };

    let seed = egg.palette_seed ^ (egg.id << 16) ^ 0xA7C4;
    let name = generated_name(seed);
    let monster_id = state
        .monster_roster
        .add_monster(name.clone(), species, seed);

    if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
        let trait_seed = if mutation_active { 0 } else { seed };
        let element_pool = match inherited {
            Some(inheritance) if !inheritance.element_options.is_empty() => {
                inheritance.element_options.as_slice()
            }
            _ => egg_type.element_bias.as_slice(),
        };
        let temperament_pool = match inherited {
            Some(inheritance) if !inheritance.temperament_options.is_empty() => {
                inheritance.temperament_options.as_slice()
            }
            _ => egg_type.temperament_bias.as_slice(),
        };
        if let Some(element) = select_by_seed(trait_seed, element_pool) {
            monster.element = *element;
        }
        if let Some(temperament) = select_by_seed(trait_seed.rotate_left(7), temperament_pool) {
            monster.temperament = *temperament;
        }
        if let Some(inheritance) = inherited {
            if let Some(passive) =
                select_by_seed(trait_seed.rotate_left(13), &inheritance.passive_options)
            {
                monster.passive = *passive;
            }
            apply_lineage_quality(monster, inheritance.lineage_quality);
        }
        monster.refresh_art_profile(species);
        if let Some(inheritance) = inherited {
            if !inheritance.art_profile.lineage_code.is_empty() {
                monster.art_profile.lineage_code = inheritance.art_profile.lineage_code.clone();
            }
            if inheritance.mutated {
                monster.art_profile.markings = format!(
                    "{} with asymmetric tower-mutation streaks",
                    monster.art_profile.markings
                );
            }
        }
    }

    let lineage_note = if mutation_active {
        " A tower-borne mutation shows in its markings."
    } else if inherited.is_some() {
        " Its inherited traits carried through."
    } else {
        ""
    };
    let summary = format!(
        "{} hatched into {} the {}.{}",
        egg_type.name, name, species.name, lineage_note
    );
    state.activity_log.add(state.day, summary.clone());

    EggResult { summary }
}

fn select_species_id<'a>(seed: u64, species_ids: &'a [String]) -> Option<&'a str> {
    species_ids
        .get((seed as usize) % species_ids.len().max(1))
        .map(String::as_str)
}

fn select_by_seed<T>(seed: u64, values: &[T]) -> Option<&T> {
    values.get((seed as usize) % values.len().max(1))
}

fn generated_name(seed: u64) -> String {
    const NAMES: [&str; 12] = [
        "Momo", "Nix", "Tula", "Bramble", "Pico", "Lumi", "Rook", "Fenn", "Saff", "Peb", "Clover",
        "Kip",
    ];
    NAMES[(seed as usize) % NAMES.len()].to_owned()
}

fn apply_lineage_quality(monster: &mut crate::state::MonsterInstance, quality: u32) {
    let bonus = quality.saturating_sub(1).min(4);
    if bonus == 0 {
        return;
    }

    monster.bond += bonus;
    monster.max_hp += bonus as i32;
    match monster.role {
        MonsterRole::Scout => monster.speed += bonus as i32,
        MonsterRole::Tank => monster.defense += bonus as i32,
        MonsterRole::Support => monster.max_hp += bonus as i32,
        MonsterRole::Striker => monster.attack += bonus as i32,
    }
    monster.hp = monster.max_hp;
}

fn cost_text(data: &GameData, cost: &[(String, i32)]) -> String {
    cost.iter()
        .map(|(resource_id, amount)| format!("{} {}", amount, data.resource_name(resource_id)))
        .collect::<Vec<_>>()
        .join(", ")
}
