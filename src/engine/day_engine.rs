use crate::data::GameData;
use crate::engine::{job_engine, monster_engine};
use crate::state::GameState;

pub struct DayResult {
    pub summary: String,
}

pub fn sleep(state: &mut GameState, data: &GameData) -> DayResult {
    let previous_day = state.day;
    state.day += 1;

    let mut eggs_warmed = 0;
    for egg in &mut state.egg_inventory.eggs {
        if egg.days_remaining > 0 {
            egg.days_remaining -= 1;
            eggs_warmed += 1;
        }
    }

    let job_result = job_engine::run_daily_jobs(state, data);
    let recovery = monster_engine::recover_monsters(state);
    let mut summary = if eggs_warmed > 0 {
        format!(
            "Day {} ends. {} egg(s) warmed overnight and the monsters recovered.",
            previous_day, eggs_warmed
        )
    } else {
        format!(
            "Day {} ends. The camp rests, and every monster wakes recovered.",
            previous_day
        )
    };
    if !job_result.summary.is_empty() {
        summary.push(' ');
        summary.push_str(&job_result.summary);
    }
    if recovery.injuries_healed > 0 {
        summary.push_str(&format!(
            " {} injury recovery complete.",
            recovery.injuries_healed
        ));
    } else if recovery.fatigue_reduced > 0 {
        summary.push_str(&format!(
            " {} tired monster(s) shook off strain.",
            recovery.fatigue_reduced
        ));
    }
    state.activity_log.add(state.day, summary.clone());

    DayResult { summary }
}
