use crate::data::GameData;
use crate::state::GameState;

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
