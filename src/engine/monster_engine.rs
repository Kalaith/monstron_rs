use crate::data::GameData;
use crate::state::{DailyCommitment, GameState, MonsterInstance};

const FATIGUE_CAP: u32 = 6;

pub struct MonsterResult {
    pub summary: String,
}

pub fn toggle_party_member(
    state: &mut GameState,
    data: &GameData,
    monster_id: u64,
) -> MonsterResult {
    let Some(monster) = state.monster_roster.monster(monster_id) else {
        return MonsterResult {
            summary: "That monster is not in the roster.".to_owned(),
        };
    };
    let monster_name = monster.name.clone();
    let species_name = data
        .species(&monster.species_id)
        .map(|species| species.name.clone())
        .unwrap_or_else(|| monster.species_id.clone());

    if let Some(slot_index) = state
        .monster_roster
        .party_slots
        .iter()
        .position(|slot| slot.is_some_and(|id| id == monster_id))
    {
        state.monster_roster.remove_from_party(slot_index);
        let summary = format!("Benched {} the {}.", monster_name, species_name);
        state.activity_log.add(state.day, summary.clone());
        return MonsterResult { summary };
    }

    if monster.is_injured() {
        return MonsterResult {
            summary: format!(
                "{monster_name} needs {} more day(s) of rest.",
                monster.condition.injury_days
            ),
        };
    }

    if state.town.monster_job(monster_id).is_some() {
        return MonsterResult {
            summary: format!("{monster_name} is committed to town work today."),
        };
    }

    if monster.condition.commitment != DailyCommitment::Free {
        return MonsterResult {
            summary: format!(
                "{} is already committed to {} today.",
                monster_name,
                commitment_label(monster.condition.commitment)
            ),
        };
    }

    match state.monster_roster.assign_to_party(monster_id) {
        Ok(slot_index) => {
            let summary = format!(
                "Assigned {} the {} to party slot {}.",
                monster_name,
                species_name,
                slot_index + 1
            );
            state.activity_log.add(state.day, summary.clone());
            MonsterResult { summary }
        }
        Err(error) => MonsterResult { summary: error },
    }
}

pub fn remove_party_slot(state: &mut GameState, slot_index: usize) -> MonsterResult {
    match state.monster_roster.remove_from_party(slot_index) {
        Some(monster_id) => {
            let monster_name = state
                .monster_roster
                .monster(monster_id)
                .map(|monster| monster.name.clone())
                .unwrap_or_else(|| format!("Monster #{monster_id}"));
            let summary = format!("Removed {monster_name} from party slot {}.", slot_index + 1);
            state.activity_log.add(state.day, summary.clone());
            MonsterResult { summary }
        }
        None => MonsterResult {
            summary: "That party slot is already empty.".to_owned(),
        },
    }
}

pub struct RecoveryResult {
    pub fatigue_reduced: usize,
    pub injuries_healed: usize,
    pub rested: usize,
}

pub fn recover_monsters(state: &mut GameState) -> RecoveryResult {
    let assigned_ids = state
        .town
        .assignments
        .iter()
        .map(|assignment| assignment.monster_id)
        .collect::<Vec<_>>();
    let mut fatigue_reduced = 0;
    let mut injuries_healed = 0;
    let mut rested = 0;

    for monster in &mut state.monster_roster.monsters {
        monster.hp = monster.max_hp;
        let is_working = assigned_ids.contains(&monster.id);
        let is_resting = !is_working
            && matches!(
                monster.condition.commitment,
                DailyCommitment::Free | DailyCommitment::Rest
            );
        let fatigue_recovery = if is_resting {
            2
        } else if is_working {
            0
        } else {
            1
        };

        if monster.condition.fatigue > 0 && fatigue_recovery > 0 {
            monster.condition.fatigue = monster.condition.fatigue.saturating_sub(fatigue_recovery);
            fatigue_reduced += 1;
        }

        if monster.condition.injury_days > 0 && is_resting {
            monster.condition.injury_days -= 1;
            if monster.condition.injury_days == 0 {
                injuries_healed += 1;
            }
        }

        if is_resting {
            monster.bond += 1;
            rested += 1;
        }
        monster.condition.commitment = DailyCommitment::Free;
    }

    RecoveryResult {
        fatigue_reduced,
        injuries_healed,
        rested,
    }
}

pub fn add_fatigue(monster: &mut MonsterInstance, amount: u32) {
    monster.condition.fatigue = (monster.condition.fatigue + amount).min(FATIGUE_CAP);
}

pub fn add_injury(monster: &mut MonsterInstance, days: u32) {
    monster.condition.injury_days = monster.condition.injury_days.max(days);
    monster.hp = monster.hp.max(1);
    add_fatigue(monster, 2);
}

pub fn condition_label(monster: &MonsterInstance) -> String {
    if monster.condition.injury_days > 0 {
        format!("Injured {}d", monster.condition.injury_days)
    } else if monster.condition.fatigue >= 5 {
        "Exhausted".to_owned()
    } else if monster.condition.fatigue >= 3 {
        "Tired".to_owned()
    } else if monster.condition.fatigue > 0 {
        "Winded".to_owned()
    } else {
        "Ready".to_owned()
    }
}

pub fn commitment_label(commitment: DailyCommitment) -> &'static str {
    match commitment {
        DailyCommitment::Free => "free",
        DailyCommitment::Tower => "tower exploration",
        DailyCommitment::Breeding => "breeding care",
        DailyCommitment::Rest => "rest",
    }
}

pub fn daily_plan_label(state: &GameState, monster: &MonsterInstance) -> String {
    if let Some(job) = state.town.monster_job(monster.id) {
        return crate::engine::job_engine::job_label(job).to_owned();
    }
    match monster.condition.commitment {
        DailyCommitment::Free => "Open".to_owned(),
        DailyCommitment::Tower => "Tower".to_owned(),
        DailyCommitment::Breeding => "Breeding".to_owned(),
        DailyCommitment::Rest => "Rest".to_owned(),
    }
}

pub fn can_take_daily_action(state: &GameState, monster: &MonsterInstance) -> Result<(), String> {
    if monster.is_injured() {
        return Err(format!(
            "{} needs {} more day(s) of rest.",
            monster.name, monster.condition.injury_days
        ));
    }
    if state.town.monster_job(monster.id).is_some() {
        return Err(format!("{} is committed to town work today.", monster.name));
    }
    if monster.condition.commitment != DailyCommitment::Free {
        return Err(format!(
            "{} is already committed to {} today.",
            monster.name,
            commitment_label(monster.condition.commitment)
        ));
    }
    Ok(())
}

pub fn mark_commitment(
    state: &mut GameState,
    monster_id: u64,
    commitment: DailyCommitment,
) -> bool {
    let Some(monster) = state.monster_roster.monster_mut(monster_id) else {
        return false;
    };
    monster.condition.commitment = commitment;
    true
}

#[cfg(test)]
mod tests {
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
}
