use crate::state::GameState;

pub struct DayResult {
    pub summary: String,
}

pub fn sleep(state: &mut GameState) -> DayResult {
    let previous_day = state.day;
    state.day += 1;

    for monster in &mut state.monster_roster.monsters {
        monster.hp = monster.max_hp;
        monster.bond += 1;
    }

    let mut eggs_warmed = 0;
    for egg in &mut state.egg_inventory.eggs {
        if egg.days_remaining > 0 {
            egg.days_remaining -= 1;
            eggs_warmed += 1;
        }
    }

    let summary = if eggs_warmed > 0 {
        format!(
            "Day {} ends. {} egg(s) warmed overnight and the party recovered.",
            previous_day, eggs_warmed
        )
    } else {
        format!(
            "Day {} ends. The camp rests, and every monster wakes fully recovered.",
            previous_day
        )
    };
    state.activity_log.add(state.day, summary.clone());

    DayResult { summary }
}
