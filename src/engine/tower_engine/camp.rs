use super::anomalies::anomaly_effect;
use super::{result, TowerResult};
use crate::data::{GameData, TowerAnomalyEffect};
use crate::state::{GameState, TowerRoomKind, TowerRunState};

pub fn camp_party(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(run) = &state.tower_run else {
        return result("No tower run is active. Tap Town to choose a run.");
    };
    if run.camp_cooldown > 0 {
        return result(format!(
            "The party must travel {} more step(s) before camping again. Tap EXPLORE or RETREAT.",
            run.camp_cooldown
        ));
    }

    let mending_lights = anomaly_effect(run, data) == Some(TowerAnomalyEffect::MendingLights);
    let sheltered = camp_sheltered(run);
    let healing = 3 + if mending_lights { 2 } else { 0 } + if sheltered { 2 } else { 0 };
    let party_ids = state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let mut total_healed = 0;
    for monster_id in party_ids {
        if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
            let before = monster.hp;
            monster.hp = (monster.hp + healing).min(monster.max_hp);
            total_healed += monster.hp - before;
        }
    }

    let run = state.tower_run.as_mut().expect("tower run checked above");
    let pressure_capacity = 2 + u32::from(sheltered);
    let pressure_reduced = run.pressure.min(pressure_capacity);
    run.pressure -= pressure_reduced;
    run.camp_cooldown = 8_u32
        .saturating_sub(if mending_lights { 2 } else { 0 })
        .saturating_sub(if sheltered { 2 } else { 0 })
        .max(4);
    let camp_label = if sheltered {
        "the marked shelter"
    } else {
        "a brief corridor camp"
    };
    let summary = format!(
        "The party rests in {camp_label}: {total_healed} total HP restored, pressure reduced by {pressure_reduced}, and CAMP readies in {} steps.",
        run.camp_cooldown
    );
    run.add_event(summary.clone());
    result(summary)
}

pub fn camp_sheltered(run: &TowerRunState) -> bool {
    current_room_index(run)
        .is_some_and(|index| index == 0 || run.map.room_kind(index) == TowerRoomKind::Camp)
}

pub(super) fn camp_room_center(run: &TowerRunState) -> Option<(u32, u32)> {
    run.map
        .rooms
        .iter()
        .enumerate()
        .filter(|(index, _)| *index == 0 || run.map.room_kind(*index) == TowerRoomKind::Camp)
        .map(|(_, room)| room.center())
        .min_by_key(|center| {
            run.map.player_x.abs_diff(center.0) + run.map.player_y.abs_diff(center.1)
        })
}

fn current_room_index(run: &TowerRunState) -> Option<usize> {
    run.map.rooms.iter().position(|room| {
        run.map.player_x >= room.start_x
            && run.map.player_x < room.start_x + room.width
            && run.map.player_y >= room.start_y
            && run.map.player_y < room.start_y + room.height
    })
}

#[cfg(test)]
mod tests;
