use crate::data::{GameData, TowerContractDefinition, TowerContractTarget};
use crate::state::TowerRunState;

#[cfg(test)]
mod tests;

pub(super) fn refresh_contract(run: &mut TowerRunState, data: &GameData) -> Option<String> {
    if run.contract_id.is_empty() {
        run.contract_id = run.goal.contract_id().to_owned();
    }
    if run.contract_complete {
        return None;
    }
    let contract = data.tower_contract(&run.contract_id)?;
    if progress(run, contract) < contract.target_amount {
        return None;
    }

    run.contract_complete = true;
    let mut reward_labels = Vec::new();
    for reward in &contract.rewards {
        run.add_cargo(&reward.resource_id, reward.amount);
        reward_labels.push(format!(
            "{} {}",
            reward.amount,
            data.resource_name(&reward.resource_id)
        ));
    }
    let summary = format!(
        "Contract complete — {}. Bonus secured: {}.",
        contract.name,
        reward_labels.join(", ")
    );
    run.add_event(summary.clone());
    Some(summary)
}

pub fn contract_progress<'a>(
    run: &TowerRunState,
    data: &'a GameData,
) -> Option<(u32, &'a TowerContractDefinition)> {
    let contract_id = if run.contract_id.is_empty() {
        run.goal.contract_id()
    } else {
        &run.contract_id
    };
    let contract = data.tower_contract(contract_id)?;
    Some((progress(run, contract), contract))
}

fn progress(run: &TowerRunState, contract: &TowerContractDefinition) -> u32 {
    match contract.target {
        TowerContractTarget::Eggs => run.found_eggs.len() as u32,
        TowerContractTarget::Cargo => run.cargo_amount() as u32,
        TowerContractTarget::Steps => run.rooms_explored,
        TowerContractTarget::Floors => run.stats.floors_descended,
        TowerContractTarget::Landmarks => run.stats.landmarks_resolved,
        TowerContractTarget::HazardsCountered => run.stats.hazards_countered,
    }
}
