use crate::data::GameData;
use crate::state::{GameState, MonsterInstance};

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
}

pub fn recover_monsters(state: &mut GameState) -> RecoveryResult {
    let mut fatigue_reduced = 0;
    let mut injuries_healed = 0;

    for monster in &mut state.monster_roster.monsters {
        monster.hp = monster.max_hp;

        if monster.condition.fatigue > 0 {
            monster.condition.fatigue = monster.condition.fatigue.saturating_sub(2);
            fatigue_reduced += 1;
        }

        if monster.condition.injury_days > 0 {
            monster.condition.injury_days -= 1;
            if monster.condition.injury_days == 0 {
                injuries_healed += 1;
            }
        }

        monster.bond += 1;
    }

    RecoveryResult {
        fatigue_reduced,
        injuries_healed,
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
        assert_eq!(recovered.condition.fatigue, 3);
        assert_eq!(recovered.condition.injury_days, 0);
        assert_eq!(recovered.hp, recovered.max_hp);
    }
}
