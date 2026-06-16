use crate::data::{GameData, MonsterRole, Temperament};
use crate::engine::town_engine;
use crate::state::{EggCareFocus, EggInstance, GameState};

const HATCHERY_ID: &str = "hatchery";

pub struct EggResult {
    pub summary: String,
}

pub fn care_for_egg(
    state: &mut GameState,
    data: &GameData,
    egg_id: u64,
    care_focus: EggCareFocus,
) -> EggResult {
    if state.town.building_level(HATCHERY_ID) == 0 {
        return EggResult {
            summary: "Build the hatchery before caring for eggs.".to_owned(),
        };
    }

    let Some(egg) = state.egg_inventory.eggs.iter().find(|egg| egg.id == egg_id) else {
        return EggResult {
            summary: "That egg is no longer in the hatchery.".to_owned(),
        };
    };

    if egg.last_care_day == state.day {
        return EggResult {
            summary: "That egg has already received care today.".to_owned(),
        };
    }

    if care_focus == EggCareFocus::Warm && egg.days_remaining == 0 {
        return EggResult {
            summary: "That egg is already ready to hatch.".to_owned(),
        };
    }

    let cost = care_cost(care_focus);
    if !cost.is_empty() {
        if let Err(missing) = state.resources.spend(&cost) {
            return EggResult {
                summary: format!("{} needs {}.", care_focus, cost_text(data, &missing)),
            };
        }
    }

    let day = state.day;
    let Some(egg) = state.egg_inventory.egg_mut(egg_id) else {
        return EggResult {
            summary: "That egg is no longer in the hatchery.".to_owned(),
        };
    };

    apply_care(egg, care_focus, day);
    let egg_name = data
        .egg_type(&egg.egg_type_id)
        .map(|egg_type| egg_type.name.as_str())
        .unwrap_or(egg.egg_type_id.as_str());
    let summary = format!("{care_focus} {} #{}. {}", egg_name, egg.id, care_bonus(egg));
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

    if !town_engine::has_monster_capacity(state) {
        let current = state.monster_roster.monsters.len();
        let capacity = town_engine::monster_capacity(state);
        return EggResult {
            summary: format!(
                "Stable capacity is full ({current}/{capacity}). Build or upgrade the Stable before hatching."
            ),
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
    let mutation_active =
        inherited.is_some_and(|inheritance| inheritance.mutated) && !egg.stabilised;
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
        if egg.care_focus == EggCareFocus::Soothe && egg.care_points > 0 {
            if let Some(temperament) = select_by_seed(
                trait_seed.rotate_left(19),
                &[
                    Temperament::Gentle,
                    Temperament::Loyal,
                    Temperament::Patient,
                ],
            ) {
                monster.temperament = *temperament;
            }
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

    let lineage_note = if inherited.is_some_and(|inheritance| inheritance.mutated) && egg.stabilised
    {
        " Care stabilised its tower mutation."
    } else if mutation_active {
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

pub fn likely_species_text(egg: &EggInstance, data: &GameData) -> String {
    let species_options: Option<&[String]> = egg
        .inheritance
        .as_ref()
        .filter(|inheritance| !inheritance.species_options.is_empty())
        .map(|inheritance| inheritance.species_options.as_slice())
        .or_else(|| {
            data.egg_type(&egg.egg_type_id)
                .map(|egg_type| egg_type.possible_species.as_slice())
        });
    let Some(species_options) = species_options else {
        return "Likely: unknown".to_owned();
    };
    let names = species_options
        .iter()
        .take(2)
        .map(|species_id| {
            data.species(species_id)
                .map(|species| species.name.clone())
                .unwrap_or_else(|| species_id.clone())
        })
        .collect::<Vec<_>>()
        .join(" / ");
    format!("Likely: {names}")
}

pub fn trait_preview_text(egg: &EggInstance, data: &GameData) -> String {
    if !egg.studied && egg.inheritance.is_none() {
        return "Traits: study for hints".to_owned();
    }
    if let Some(inheritance) = &egg.inheritance {
        let element = inheritance
            .element_options
            .iter()
            .take(2)
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("/");
        let temperament = inheritance
            .temperament_options
            .iter()
            .take(2)
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("/");
        let mutation = if inheritance.mutated && !egg.stabilised {
            "  Mutation risk"
        } else if inheritance.mutated {
            "  Stabilised"
        } else {
            ""
        };
        return format!("Traits: {element} {temperament}{mutation}");
    }
    let Some(egg_type) = data.egg_type(&egg.egg_type_id) else {
        return "Traits: unknown".to_owned();
    };
    let element = egg_type
        .element_bias
        .iter()
        .take(2)
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("/");
    let temperament = egg_type
        .temperament_bias
        .iter()
        .take(2)
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("/");
    format!("Traits: {element} {temperament}")
}

pub fn care_bonus(egg: &EggInstance) -> String {
    match egg.care_focus {
        EggCareFocus::None => "No care bonus yet.".to_owned(),
        EggCareFocus::Warm => format!("Care: warmed; {} day(s) remain.", egg.days_remaining),
        EggCareFocus::Soothe => "Care: soothed; better gentle, loyal, or patient odds.".to_owned(),
        EggCareFocus::Study => "Care: studied; species and trait hints improved.".to_owned(),
        EggCareFocus::Stabilise => "Care: stabilised; mutation risk suppressed.".to_owned(),
    }
}

fn care_cost(care_focus: EggCareFocus) -> Vec<(String, i32)> {
    match care_focus {
        EggCareFocus::Warm | EggCareFocus::Soothe | EggCareFocus::Stabilise => {
            vec![("herbs".to_owned(), 1)]
        }
        EggCareFocus::Study | EggCareFocus::None => Vec::new(),
    }
}

fn apply_care(egg: &mut EggInstance, care_focus: EggCareFocus, day: u32) {
    egg.care_focus = care_focus;
    egg.care_points += 1;
    egg.last_care_day = day;
    match care_focus {
        EggCareFocus::Warm => {
            egg.days_remaining = egg.days_remaining.saturating_sub(1);
        }
        EggCareFocus::Study => {
            egg.studied = true;
        }
        EggCareFocus::Stabilise => {
            egg.stabilised = true;
        }
        EggCareFocus::Soothe | EggCareFocus::None => {}
    }
}

fn select_species_id(seed: u64, species_ids: &[String]) -> Option<&str> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameDataLoader;

    #[test]
    fn stable_capacity_blocks_hatching_without_consuming_egg() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);
        state.town.set_building_level("hatchery", 1);

        let rootling = data.species("rootling").expect("rootling should exist");
        let rillfin = data.species("rillfin").expect("rillfin should exist");
        state
            .monster_roster
            .add_monster("Root".to_owned(), rootling, 0x1001);
        state
            .monster_roster
            .add_monster("Ripple".to_owned(), rillfin, 0x1002);
        let egg_id = state
            .egg_inventory
            .add_egg("mossy_egg".to_owned(), 0, 1, 0x2001);

        let result = hatch_egg(&mut state, &data, egg_id);

        assert!(result.summary.contains("Stable capacity is full"));
        assert_eq!(state.monster_roster.monsters.len(), 3);
        assert!(state.egg_inventory.eggs.iter().any(|egg| egg.id == egg_id));
    }

    #[test]
    fn upgraded_stable_allows_hatching_past_base_capacity() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);
        state.town.set_building_level("hatchery", 1);
        state.town.set_building_level("stable", 1);

        let rootling = data.species("rootling").expect("rootling should exist");
        let rillfin = data.species("rillfin").expect("rillfin should exist");
        state
            .monster_roster
            .add_monster("Root".to_owned(), rootling, 0x1001);
        state
            .monster_roster
            .add_monster("Ripple".to_owned(), rillfin, 0x1002);
        let egg_id = state
            .egg_inventory
            .add_egg("mossy_egg".to_owned(), 0, 1, 0x2001);

        let result = hatch_egg(&mut state, &data, egg_id);

        assert!(result.summary.contains("hatched"));
        assert_eq!(state.monster_roster.monsters.len(), 4);
        assert!(state.egg_inventory.eggs.is_empty());
    }
}
