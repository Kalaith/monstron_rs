use super::discovery::record_visible_discoveries;
use super::{ensure_map, result, with_contract_refresh, TowerResult};
use crate::data::GameData;
use crate::state::{GameState, TowerMapState, TowerRoomKind, TowerRunGoal, TowerTileVisibility};

pub(super) struct SurveyReveal {
    pub center: (u32, u32),
    pub signatures: usize,
    pub secrets: usize,
}

pub fn survey_floor(state: &mut GameState, data: &GameData) -> TowerResult {
    ensure_map(state, data);
    let Some(run) = &state.tower_run else {
        return result("No tower run is active. Tap Town to choose a run.");
    };
    if run.pressure >= run.pressure_limit {
        return result("The tower is fully awake. Tap CAMP or RETREAT before surveying.");
    }
    if run.survey_charges == 0 {
        return result(
            "No survey flares remain on this floor. Tap EXPLORE or descend for a fresh kit.",
        );
    }

    let reveal = {
        let run = state.tower_run.as_mut().expect("tower run checked above");
        let Some(reveal) = reveal_hidden_room_for_goal(&mut run.map, run.goal) else {
            return result("Every room on this floor is already charted. Tap EXPLORE or RETREAT.");
        };
        run.survey_charges -= 1;
        run.pressure = run.pressure.saturating_add(1).min(run.pressure_limit);
        reveal
    };
    record_visible_discoveries(state, data, None);
    let signature_label = match reveal.signatures {
        0 => "no visible signatures".to_owned(),
        1 => "1 visible signature".to_owned(),
        count => format!("{count} visible signatures"),
    };
    let secret_label = match reveal.secrets {
        0 => String::new(),
        1 => " The flare exposes 1 concealed cache.".to_owned(),
        count => format!(" The flare exposes {count} concealed caches."),
    };
    let summary = format!(
        "A survey flare charts a hidden room near grid {}, {} with {}. Pressure rises by 1; tap the revealed room to route toward it.",
        reveal.center.0 + 1,
        reveal.center.1 + 1,
        signature_label
    );
    let summary = format!("{summary}{secret_label}");
    if let Some(run) = &mut state.tower_run {
        run.add_event(summary.clone());
    }
    with_contract_refresh(result(summary), state, data)
}

pub(super) fn reveal_hidden_room_for_goal(
    map: &mut TowerMapState,
    goal: TowerRunGoal,
) -> Option<SurveyReveal> {
    map.ensure_visibility();
    let room = map
        .rooms
        .iter()
        .enumerate()
        .filter(|(_, room)| {
            let center = room.center();
            map.visibility_at(center.0, center.1) == TowerTileVisibility::Hidden
        })
        .min_by_key(|(index, room)| {
            let center = room.center();
            (
                survey_priority(goal, map.room_kind(*index)),
                map.player_x.abs_diff(center.0) + map.player_y.abs_diff(center.1),
            )
        })?
        .1
        .to_owned();

    let max_x = (room.start_x + room.width).min(map.width);
    let max_y = (room.start_y + room.height).min(map.height);
    for y in room.start_y..max_y {
        for x in room.start_x..max_x {
            if map.is_passable(x, y) {
                map.set_visibility(x, y, TowerTileVisibility::Visible);
            }
        }
    }
    let mut secrets = 0;
    for object in &mut map.objects {
        if object.kind == crate::state::TowerMapObjectKind::SecretCache
            && object.x >= room.start_x
            && object.x < max_x
            && object.y >= room.start_y
            && object.y < max_y
        {
            object.revealed = true;
            secrets += 1;
        }
    }
    let signatures = map
        .objects
        .iter()
        .filter(|object| {
            object.x >= room.start_x
                && object.x < max_x
                && object.y >= room.start_y
                && object.y < max_y
        })
        .count();
    Some(SurveyReveal {
        center: room.center(),
        signatures,
        secrets,
    })
}

fn survey_priority(goal: TowerRunGoal, kind: TowerRoomKind) -> u8 {
    match goal {
        TowerRunGoal::EggHunt if kind == TowerRoomKind::Nest => 0,
        TowerRunGoal::Salvage if kind == TowerRoomKind::Cache => 0,
        TowerRunGoal::PushDeeper if kind == TowerRoomKind::Traversal => 0,
        TowerRunGoal::SafeRun
            if matches!(
                kind,
                TowerRoomKind::Camp
                    | TowerRoomKind::Cache
                    | TowerRoomKind::Nest
                    | TowerRoomKind::Landmark
            ) =>
        {
            0
        }
        TowerRunGoal::SafeRun
            if matches!(kind, TowerRoomKind::Encounter | TowerRoomKind::Hazard) =>
        {
            2
        }
        _ => 1,
    }
}

#[cfg(test)]
mod tests;
