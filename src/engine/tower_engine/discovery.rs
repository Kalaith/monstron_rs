use crate::data::GameData;
use crate::state::{GameState, TowerMapObject, TowerMapObjectKind};

pub(super) fn record_visible_discoveries(
    state: &mut GameState,
    data: &GameData,
    extra: Option<&TowerMapObject>,
) {
    let mut candidates = state
        .tower_run
        .as_ref()
        .map(|run| {
            run.map
                .objects
                .iter()
                .filter(|object| run.map.is_visible(object.x, object.y))
                .filter_map(candidate)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(candidate) = extra.and_then(candidate) {
        candidates.push(candidate);
    }

    let mut names = Vec::new();
    for (kind, id) in candidates {
        let discovered = match kind {
            DiscoveryKind::Enemy => state.tower_discoveries.discover_enemy(&id),
            DiscoveryKind::Location => state.tower_discoveries.discover_special_location(&id),
            DiscoveryKind::Hazard => state.tower_discoveries.discover_hazard(&id),
        };
        if discovered {
            names.push(discovery_name(data, kind, &id));
        }
    }
    if !names.is_empty() {
        let summary = format!("Field guide updated: {}.", names.join(", "));
        if let Some(run) = &mut state.tower_run {
            run.add_event(summary);
        }
    }
}

pub(super) fn record_enemy_discovery(state: &mut GameState, data: &GameData, enemy_id: &str) {
    if state.tower_discoveries.discover_enemy(enemy_id) {
        let name = data
            .enemy(enemy_id)
            .map(|enemy| enemy.name.as_str())
            .unwrap_or(enemy_id);
        if let Some(run) = &mut state.tower_run {
            run.add_event(format!("Field guide updated: {name}."));
        }
    }
}

#[derive(Clone, Copy)]
enum DiscoveryKind {
    Enemy,
    Location,
    Hazard,
}

fn candidate(object: &TowerMapObject) -> Option<(DiscoveryKind, String)> {
    match object.kind {
        TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss => {
            Some((DiscoveryKind::Enemy, object.enemy_id.clone()))
        }
        TowerMapObjectKind::SpecialLocation => {
            Some((DiscoveryKind::Location, object.special_location_id.clone()))
        }
        TowerMapObjectKind::Hazard => Some((DiscoveryKind::Hazard, object.hazard_id.clone())),
        TowerMapObjectKind::Loot
        | TowerMapObjectKind::SecretCache
        | TowerMapObjectKind::Egg
        | TowerMapObjectKind::Stairs
        | TowerMapObjectKind::Exit => None,
    }
}

fn discovery_name(data: &GameData, kind: DiscoveryKind, id: &str) -> String {
    match kind {
        DiscoveryKind::Enemy => data.enemy(id).map(|entry| entry.name.clone()),
        DiscoveryKind::Location => data
            .tower_special_location(id)
            .map(|entry| entry.name.clone()),
        DiscoveryKind::Hazard => data.tower_hazard(id).map(|entry| entry.name.clone()),
    }
    .unwrap_or_else(|| id.to_owned())
}
