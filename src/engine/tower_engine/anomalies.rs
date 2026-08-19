use crate::data::{GameData, TowerAnomalyEffect};
use crate::state::TowerRunState;

pub(super) fn select_anomaly_id(data: &GameData, floor: u32, seed: u64) -> String {
    let eligible = data
        .tower_anomalies
        .iter()
        .filter(|anomaly| anomaly.min_floor <= floor && anomaly.max_floor >= floor)
        .collect::<Vec<_>>();
    if eligible.is_empty() {
        return String::new();
    }
    let index =
        (seed.rotate_left(13) ^ u64::from(floor).wrapping_mul(0x9E37)) as usize % eligible.len();
    eligible[index].id.clone()
}

pub(super) fn anomaly_effect(run: &TowerRunState, data: &GameData) -> Option<TowerAnomalyEffect> {
    data.tower_anomaly(&run.anomaly_id)
        .map(|anomaly| anomaly.effect)
}

#[cfg(test)]
mod tests;
