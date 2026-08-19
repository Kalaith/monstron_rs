use super::map_objects::spawn_pressure_enemy;
use crate::data::GameData;
use crate::state::GameState;

pub(super) fn refresh_pressure(state: &mut GameState, data: &GameData) -> Option<String> {
    let run = state.tower_run.as_mut()?;
    let next_stage = pressure_stage(run.pressure, run.pressure_limit);
    if next_stage <= run.pressure_stage {
        return None;
    }

    let previous_stage = run.pressure_stage;
    run.pressure_stage = next_stage;
    let mut messages = Vec::new();
    if previous_stage < 1 && next_stage >= 1 {
        messages.push("The tower stirs; distant rooms answer the party's footsteps.".to_owned());
    }
    if previous_stage < 2 && next_stage >= 2 {
        let seed = run.map.seed
            ^ u64::from(run.pressure).wrapping_mul(0x9E37)
            ^ u64::from(run.rooms_explored).rotate_left(17);
        if let Some(enemy_id) = spawn_pressure_enemy(&mut run.map, data, seed) {
            let enemy_name = data
                .enemy(&enemy_id)
                .map(|enemy| enemy.name.as_str())
                .unwrap_or(&enemy_id);
            messages.push(format!(
                "High pressure draws a wandering {enemy_name} onto the map."
            ));
        } else {
            messages.push("High pressure sends something searching through the floor.".to_owned());
        }
    }
    if previous_stage < 3 && next_stage >= 3 {
        messages.push("The tower is fully awake. CAMP or RETREAT before moving again.".to_owned());
    }

    let summary = messages.join(" ");
    run.add_event(summary.clone());
    Some(summary)
}

fn pressure_stage(pressure: u32, limit: u32) -> u8 {
    if limit == 0 || pressure == 0 {
        0
    } else if pressure >= limit {
        3
    } else if pressure.saturating_mul(3) >= limit.saturating_mul(2) {
        2
    } else if pressure.saturating_mul(3) >= limit {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests;
