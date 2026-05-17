use crate::data::{GameData, MonsterRole, PassiveSkill, TownSkill};
use crate::engine::monster_engine;
use crate::state::DailyCommitment;
use crate::state::{GameState, MonsterInstance, TownJobKind};

const WORKSHOP_ID: &str = "workshop";

pub struct JobResult {
    pub summary: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct JobYield {
    resources: Vec<(String, i32)>,
    warmed_eggs: u32,
}

pub fn assign_job(
    state: &mut GameState,
    data: &GameData,
    monster_id: u64,
    job: TownJobKind,
) -> JobResult {
    if state.town.building_level(WORKSHOP_ID) == 0 {
        return JobResult {
            summary: "Build the workshop before assigning town jobs.".to_owned(),
        };
    }

    let Some(monster) = state.monster_roster.monster(monster_id) else {
        return JobResult {
            summary: "That monster is no longer in the roster.".to_owned(),
        };
    };
    let monster_name = monster.name.clone();
    let job_name = job_label(job);
    let species_name = data
        .species(&monster.species_id)
        .map(|species| species.name.as_str())
        .unwrap_or(monster.species_id.as_str());

    if monster.is_injured() {
        return JobResult {
            summary: format!(
                "{} needs {} more day(s) of rest.",
                monster.name, monster.condition.injury_days
            ),
        };
    }
    if monster.condition.commitment != DailyCommitment::Free {
        return JobResult {
            summary: format!(
                "{} is already committed to {} today.",
                monster.name,
                monster_engine::commitment_label(monster.condition.commitment)
            ),
        };
    }

    state.town.set_monster_job(monster_id, job);
    let summary = format!("{monster_name} the {species_name} is assigned to {job_name}.");
    state.activity_log.add(state.day, summary.clone());
    JobResult { summary }
}

pub fn clear_job(state: &mut GameState, monster_id: u64) -> JobResult {
    let Some(monster) = state.monster_roster.monster(monster_id) else {
        return JobResult {
            summary: "That monster is no longer in the roster.".to_owned(),
        };
    };
    let monster_name = monster.name.clone();

    if state.town.clear_monster_job(monster_id) {
        let summary = format!("{monster_name} is resting from town work.");
        state.activity_log.add(state.day, summary.clone());
        JobResult { summary }
    } else {
        JobResult {
            summary: format!("{monster_name} has no town job assigned."),
        }
    }
}

pub fn run_daily_jobs(state: &mut GameState, data: &GameData) -> JobResult {
    let assignments = state.town.assignments.clone();
    if assignments.is_empty() {
        return JobResult {
            summary: String::new(),
        };
    }

    let mut workers = 0;
    let mut totals: Vec<(String, i32)> = Vec::new();
    let mut warmed_eggs = 0;

    for assignment in assignments {
        let Some(monster) = state.monster_roster.monster(assignment.monster_id).cloned() else {
            state.town.clear_monster_job(assignment.monster_id);
            continue;
        };
        let result = job_yield(&monster, assignment.job);
        if result.is_empty() && monster.is_injured() {
            continue;
        }
        for (resource_id, amount) in result.resources {
            state.resources.add(&resource_id, amount);
            add_total(&mut totals, resource_id, amount);
        }
        warmed_eggs += warm_eggs(state, result.warmed_eggs);
        if let Some(worker) = state.monster_roster.monster_mut(monster.id) {
            worker.bond += 1;
            monster_engine::add_fatigue(worker, 1);
        }
        workers += 1;
    }

    if workers == 0 {
        return JobResult {
            summary: String::new(),
        };
    }

    let mut parts = Vec::new();
    if !totals.is_empty() {
        parts.push(cost_text(data, &totals));
    }
    if warmed_eggs > 0 {
        parts.push(format!("{warmed_eggs} egg warmth"));
    }

    let summary = if parts.is_empty() {
        format!("{workers} town worker(s) gained bond while helping camp.")
    } else {
        format!("Town jobs produced {}.", parts.join(", "))
    };
    JobResult { summary }
}

pub fn job_label(job: TownJobKind) -> &'static str {
    match job {
        TownJobKind::Forage => "Forage",
        TownJobKind::Quarry => "Quarry",
        TownJobKind::Workshop => "Workshop",
        TownJobKind::HatcheryCare => "Egg Care",
    }
}

pub fn job_detail(job: TownJobKind) -> &'static str {
    match job {
        TownJobKind::Forage => "Herbs and coins; scout traits boost.",
        TownJobKind::Quarry => "Stone, with ore from stone traits.",
        TownJobKind::Workshop => "Wood and stone; heat traits add ore.",
        TownJobKind::HatcheryCare => "Warms eggs overnight; bond grows.",
    }
}

pub fn job_preview(monster: &MonsterInstance, data: &GameData, job: TownJobKind) -> String {
    let result = job_yield(monster, job);
    if result.is_empty() && monster.is_injured() {
        return "Recovering; no job output.".to_owned();
    }
    let mut parts = Vec::new();
    if !result.resources.is_empty() {
        parts.push(cost_text(data, &result.resources));
    }
    if result.warmed_eggs > 0 {
        parts.push(format!("{} egg warmth", result.warmed_eggs));
    }
    if parts.is_empty() {
        "Bond +1".to_owned()
    } else {
        format!("{}; Bond +1", parts.join(", "))
    }
}

fn job_yield(monster: &MonsterInstance, job: TownJobKind) -> JobYield {
    if monster.is_injured() {
        return JobYield::default();
    }

    let mut result = JobYield::default();
    match job {
        TownJobKind::Forage => {
            add_total(&mut result.resources, "herbs".to_owned(), 2);
            add_total(&mut result.resources, "coins".to_owned(), 3);
            if monster.role == MonsterRole::Scout {
                add_total(&mut result.resources, "herbs".to_owned(), 1);
            }
            if monster.town_skill == TownSkill::Farming {
                add_total(&mut result.resources, "herbs".to_owned(), 1);
            }
            if monster.passive == PassiveSkill::FindsSmallLoot {
                add_total(&mut result.resources, "coins".to_owned(), 2);
            }
        }
        TownJobKind::Quarry => {
            add_total(&mut result.resources, "stone".to_owned(), 3);
            if monster.role == MonsterRole::Tank {
                add_total(&mut result.resources, "stone".to_owned(), 1);
            }
            if monster.passive == PassiveSkill::FindsStone {
                add_total(&mut result.resources, "ore".to_owned(), 1);
            }
        }
        TownJobKind::Workshop => {
            add_total(&mut result.resources, "wood".to_owned(), 3);
            add_total(&mut result.resources, "stone".to_owned(), 2);
            if monster.town_skill == TownSkill::WorkshopHeat
                || monster.passive == PassiveSkill::BurnsBrambles
            {
                add_total(&mut result.resources, "ore".to_owned(), 1);
            }
        }
        TownJobKind::HatcheryCare => {
            result.warmed_eggs = if monster.town_skill == TownSkill::Hatching
                || monster.town_skill == TownSkill::HatcheryHelper
            {
                2
            } else {
                1
            };
        }
    }
    apply_fatigue_to_yield(&mut result, monster.condition.fatigue);
    result
}

impl JobYield {
    fn is_empty(&self) -> bool {
        self.resources.is_empty() && self.warmed_eggs == 0
    }
}

fn apply_fatigue_to_yield(result: &mut JobYield, fatigue: u32) {
    if fatigue < 4 {
        return;
    }

    for (_, amount) in &mut result.resources {
        *amount = (*amount - 1).max(1);
    }
    if result.warmed_eggs > 1 {
        result.warmed_eggs -= 1;
    }
}

fn warm_eggs(state: &mut GameState, mut warmth: u32) -> u32 {
    let mut warmed = 0;
    for egg in &mut state.egg_inventory.eggs {
        if warmth == 0 {
            break;
        }
        if egg.days_remaining > 0 {
            egg.days_remaining -= 1;
            warmth -= 1;
            warmed += 1;
        }
    }
    warmed
}

fn add_total(totals: &mut Vec<(String, i32)>, resource_id: String, amount: i32) {
    if let Some((_, total)) = totals
        .iter_mut()
        .find(|(existing_id, _)| existing_id == &resource_id)
    {
        *total += amount;
    } else {
        totals.push((resource_id, amount));
    }
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
}
