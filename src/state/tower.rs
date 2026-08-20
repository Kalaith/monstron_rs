//! Persistent tower state, split into run, map, discovery, and goal concerns.

mod discovery;
mod goal;
mod map;
mod run;
#[cfg(test)]
mod tests;

pub use discovery::TowerDiscoveryState;
pub use goal::{survey_charges_for, TowerRunGoal};
pub use map::{
    TowerMapObject, TowerMapObjectKind, TowerMapRng, TowerMapState, TowerRoom, TowerRoomKind,
    TowerTileKind, TowerTileVisibility,
};
pub use run::{
    TowerCompletedLandmark, TowerFoundEgg, TowerPendingEvent, TowerProgress, TowerRunState,
    TowerRunStats,
};
