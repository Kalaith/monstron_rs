use super::discovery::record_enemy_discovery;
use super::{result, TowerEncounterRequest, TowerResult};
use crate::data::GameData;
use crate::state::{GameState, TowerFoundEgg, TowerTileVisibility};

pub(super) fn resolve_hazard(
    state: &mut GameState,
    data: &GameData,
    hazard_id: &str,
) -> TowerResult {
    let Some(hazard) = data.tower_hazard(hazard_id) else {
        return result("The party crosses an unrecorded tower hazard.");
    };
    let countering_monster = state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .filter_map(|id| state.monster_roster.monster(*id))
        .find(|monster| {
            hazard
                .counter_passive
                .is_some_and(|passive| passive == monster.passive)
                || hazard
                    .counter_element
                    .is_some_and(|element| element == monster.element)
        })
        .map(|monster| monster.name.clone());

    let summary = if let Some(monster_name) = countering_monster {
        let mut rewards = Vec::new();
        if let Some(run) = &mut state.tower_run {
            run.stats.hazards_countered += 1;
            for reward in &hazard.counter_rewards {
                run.add_cargo(&reward.resource_id, reward.amount);
                rewards.push(format!(
                    "{} {}",
                    reward.amount,
                    data.resource_name(&reward.resource_id)
                ));
            }
        }
        let reward_text = if rewards.is_empty() {
            String::new()
        } else {
            format!(" Recovered {}.", rewards.join(", "))
        };
        format!(
            "{monster_name} counters {} before it closes on the party.{reward_text}",
            hazard.name
        )
    } else {
        let party_ids: Vec<u64> = state
            .monster_roster
            .party_slots
            .iter()
            .flatten()
            .copied()
            .collect();
        let mut total_damage = 0;
        for monster_id in party_ids {
            if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
                let before = monster.hp;
                monster.hp = (monster.hp - hazard.damage).max(1);
                total_damage += before - monster.hp;
            }
        }
        if let Some(run) = &mut state.tower_run {
            run.pressure = run
                .pressure
                .saturating_add(hazard.pressure)
                .min(run.pressure_limit);
        }
        format!(
            "{} catches the party: {total_damage} total damage and +{} pressure.",
            hazard.name, hazard.pressure
        )
    };
    if let Some(run) = &mut state.tower_run {
        run.add_event(summary.clone());
    }
    result(summary)
}

pub(super) fn apply_tower_event(
    state: &mut GameState,
    data: &GameData,
    location_id: &str,
    event_id: &str,
) -> TowerResult {
    let Some(location) = data.tower_special_location(location_id) else {
        return result("The party finds an unrecorded tower landmark.");
    };
    let Some(event) = data.tower_event(event_id) else {
        return result(format!("{} stands silent.", location.name));
    };

    let mut reward_labels = Vec::new();
    let mut found_egg_name = None;
    if let Some(run) = &mut state.tower_run {
        for reward in &event.rewards {
            run.add_cargo(&reward.resource_id, reward.amount);
            reward_labels.push(format!(
                "{} {}",
                reward.amount,
                data.resource_name(&reward.resource_id)
            ));
        }
        if event.pressure_delta < 0 {
            run.pressure = run
                .pressure
                .saturating_sub(event.pressure_delta.unsigned_abs());
        } else {
            run.pressure = run
                .pressure
                .saturating_add(event.pressure_delta as u32)
                .min(run.pressure_limit);
        }
        if event.refresh_camp {
            run.camp_cooldown = 0;
        }
        if event.reveal_map {
            run.map.ensure_visibility();
            for (tile, visibility) in run.map.tiles.iter().zip(run.map.visibility.iter_mut()) {
                if tile.is_passable() && *visibility == TowerTileVisibility::Hidden {
                    *visibility = TowerTileVisibility::Explored;
                }
            }
        }
        if let Some(egg) = data.egg_type(&event.egg_type_id) {
            run.found_eggs.push(TowerFoundEgg {
                egg_type_id: egg.id.clone(),
                hatch_days: egg.hatch_days,
                origin_floor: run.current_floor,
                palette_seed: event_seed(run.map.seed, &event.id),
            });
            found_egg_name = Some(egg.name.clone());
        }
    }

    let party_ids: Vec<u64> = state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .copied()
        .collect();
    let mut total_healed = 0;
    for monster_id in party_ids {
        if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
            let before = monster.hp;
            monster.hp = (monster.hp + event.party_healing).min(monster.max_hp);
            total_healed += monster.hp - before;
        }
    }

    let mut summary = format!("{} — {}: {}", location.name, event.name, event.narrative);
    if !reward_labels.is_empty() {
        summary.push_str(&format!(" Recovered {}.", reward_labels.join(", ")));
    }
    if total_healed > 0 {
        summary.push_str(&format!(" Restored {total_healed} total HP."));
    }
    if let Some(egg_name) = found_egg_name {
        summary.push_str(&format!(" Recovered a {egg_name}."));
    }
    if event.reveal_map {
        summary.push_str(" The floor's passable routes are now charted.");
    }
    if event.refresh_camp {
        summary.push_str(" The party can CAMP again immediately.");
    }
    if let Some(run) = &mut state.tower_run {
        run.add_event(summary.clone());
    }

    let enemy_id = (!event.enemy_id.is_empty()).then_some(event.enemy_id.clone());
    if let Some(enemy_id) = &enemy_id {
        record_enemy_discovery(state, data, enemy_id);
    }
    TowerResult {
        summary,
        encounter: enemy_id.map(|enemy_id| TowerEncounterRequest {
            floor: state
                .tower_run
                .as_ref()
                .map(|run| run.current_floor)
                .unwrap_or(1),
            is_boss: false,
            enemy_id: Some(enemy_id),
        }),
        returned_to_town: false,
    }
}

fn event_seed(map_seed: u64, event_id: &str) -> u64 {
    event_id.bytes().fold(map_seed ^ 0xE7E7_5EED, |seed, byte| {
        seed.rotate_left(7) ^ u64::from(byte)
    })
}
