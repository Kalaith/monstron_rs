use super::interactions::apply_tower_event;
use super::{result, with_contract_refresh, TowerResult};
use crate::data::{GameData, TowerEventDefinition};
use crate::state::{GameState, TowerMapObject, TowerPendingEvent, TowerRunState};

pub(super) fn resolve_special_location(
    state: &mut GameState,
    data: &GameData,
    object: TowerMapObject,
) -> TowerResult {
    let Some(location) = data.tower_special_location(&object.special_location_id) else {
        return result("The party finds an unrecorded tower landmark.");
    };
    let mut event_ids = location.event_ids.clone();
    if let Some(index) = event_ids.iter().position(|id| id == &object.event_id) {
        event_ids.rotate_left(index);
    }
    if let Some(run) = &mut state.tower_run {
        run.pending_event = Some(TowerPendingEvent {
            special_location_id: location.id.clone(),
            event_ids,
        });
        let summary = format!(
            "Discovered {}. Choose a visible approach or tap LEAVE.",
            location.name
        );
        run.add_event(summary.clone());
        return result(summary);
    }
    result("No tower run is active. Tap Town to choose a run.")
}

pub fn choose_special_event(state: &mut GameState, data: &GameData, event_id: &str) -> TowerResult {
    let Some(pending) = state
        .tower_run
        .as_ref()
        .and_then(|run| run.pending_event.clone())
    else {
        return result("No landmark decision is waiting. Tap EXPLORE.");
    };
    if !pending
        .event_ids
        .iter()
        .any(|candidate| candidate == event_id)
    {
        return result("That landmark approach is unavailable. Tap a visible choice.");
    }
    let Some(event) = data.tower_event(event_id) else {
        return result("That landmark approach is no longer recorded.");
    };
    if let Some(requirement) = event_party_requirement_missing(state, event) {
        return result(format!(
            "{} needs a party monster with {}. Choose another approach or tap LEAVE.",
            event.name, requirement
        ));
    }
    let Some(run) = state.tower_run.as_ref() else {
        return result("No tower run is active. Tap Town to choose a run.");
    };
    if !run.can_afford_cargo(&event.cargo_costs) {
        return result(event_cost_shortfall(run, data, event));
    }
    if let Some(run) = &mut state.tower_run {
        run.spend_cargo(&event.cargo_costs);
        run.pending_event = None;
        run.stats.landmarks_resolved += 1;
    }
    let first_record = state.tower_discoveries.discover_event(event_id);
    let mut outcome = apply_tower_event(state, data, &pending.special_location_id, event_id);
    if first_record {
        let flare_added = state.tower_run.as_mut().is_some_and(|run| {
            let before = run.survey_charges;
            run.survey_charges = run.survey_charges.saturating_add(1).min(5);
            run.survey_charges > before
        });
        let note = if flare_added {
            " First approach recorded: prepared 1 bonus survey flare."
        } else {
            " First approach recorded in the Field Guide."
        };
        outcome.summary.push_str(note);
        if let Some(run) = &mut state.tower_run {
            run.add_event(note.trim().to_owned());
        }
    }
    with_contract_refresh(outcome, state, data)
}

pub fn event_choice_available(state: &GameState, data: &GameData, event_id: &str) -> bool {
    data.tower_event(event_id).is_some_and(|event| {
        event_party_requirement_missing(state, event).is_none()
            && state
                .tower_run
                .as_ref()
                .is_some_and(|run| run.can_afford_cargo(&event.cargo_costs))
    })
}

fn event_party_requirement_missing(
    state: &GameState,
    event: &TowerEventDefinition,
) -> Option<String> {
    let party = state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .filter_map(|id| state.monster_roster.monster(*id))
        .collect::<Vec<_>>();
    if let Some(passive) = event.required_passive {
        if !party.iter().any(|monster| monster.passive == passive) {
            return Some(passive.to_string());
        }
    }
    if let Some(element) = event.required_element {
        if !party.iter().any(|monster| monster.element == element) {
            return Some(format!("{element} affinity"));
        }
    }
    None
}

fn event_cost_shortfall(
    run: &TowerRunState,
    data: &GameData,
    event: &TowerEventDefinition,
) -> String {
    let missing = event
        .cargo_costs
        .iter()
        .filter_map(|cost| {
            let amount = cost.amount - run.cargo_amount_for(&cost.resource_id);
            (amount > 0).then(|| format!("{amount} {}", data.resource_name(&cost.resource_id)))
        })
        .collect::<Vec<_>>();
    format!(
        "{} needs {} more in expedition cargo. Choose another approach or tap LEAVE.",
        event.name,
        missing.join(", ")
    )
}

pub fn leave_special_event(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(pending) = state
        .tower_run
        .as_mut()
        .and_then(|run| run.pending_event.take())
    else {
        return result("No landmark decision is waiting. Tap EXPLORE.");
    };
    let location_name = data
        .tower_special_location(&pending.special_location_id)
        .map(|location| location.name.as_str())
        .unwrap_or("the landmark");
    let summary = format!("The party leaves {location_name} undisturbed.");
    if let Some(run) = &mut state.tower_run {
        run.add_event(summary.clone());
    }
    result(summary)
}

#[cfg(test)]
mod tests;
