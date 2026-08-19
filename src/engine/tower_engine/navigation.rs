use std::collections::VecDeque;

use crate::state::{TowerMapObjectKind, TowerMapState, TowerRunGoal, TowerTileVisibility};

pub(super) fn explore_direction(map: &TowerMapState, goal: TowerRunGoal) -> Option<(i32, i32)> {
    if map.is_empty() {
        return None;
    }

    let start = map_index(map, map.player_x, map.player_y)?;
    let mut queue = VecDeque::from([start]);
    let mut distances = vec![None; (map.width * map.height) as usize];
    let mut previous = vec![None; distances.len()];
    distances[start] = Some(0_u32);

    while let Some(index) = queue.pop_front() {
        let (x, y) = coordinates(map, index);
        for (next_x, next_y) in neighbors(map, x, y) {
            let Some(next) = map_index(map, next_x, next_y) else {
                continue;
            };
            if distances[next].is_some() || !map.is_passable(next_x, next_y) {
                continue;
            }
            distances[next] = Some(distances[index]? + 1);
            previous[next] = Some(index);
            queue.push_back(next);
        }
    }

    let target = best_target(map, goal, start, &distances)?;
    first_step(map, start, target, &previous)
}

fn best_target(
    map: &TowerMapState,
    goal: TowerRunGoal,
    start: usize,
    distances: &[Option<u32>],
) -> Option<usize> {
    distances
        .iter()
        .enumerate()
        .filter_map(|(index, distance)| {
            let distance = (*distance)?;
            if index == start {
                return None;
            }
            let (x, y) = coordinates(map, index);
            let bias = if map.is_discovered(x, y) {
                map.object_at(x, y)
                    .map(|object| object_bias(goal, object.kind))
            } else if map.visibility_at(x, y) == TowerTileVisibility::Hidden {
                Some(frontier_bias(goal))
            } else {
                None
            }?;
            Some((distance as i32 * 10 + bias, index))
        })
        .min_by_key(|candidate| *candidate)
        .map(|(_, index)| index)
}

fn object_bias(goal: TowerRunGoal, kind: TowerMapObjectKind) -> i32 {
    match (goal, kind) {
        (TowerRunGoal::EggHunt, TowerMapObjectKind::Egg) => -90,
        (TowerRunGoal::Salvage, TowerMapObjectKind::Loot) => -80,
        (TowerRunGoal::Scout, TowerMapObjectKind::SpecialLocation) => -80,
        (TowerRunGoal::PushDeeper, TowerMapObjectKind::Stairs) => -100,
        (TowerRunGoal::SafeRun, TowerMapObjectKind::Exit) => -50,
        (TowerRunGoal::SafeRun, TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss) => 140,
        (TowerRunGoal::SafeRun, TowerMapObjectKind::Hazard) => 170,
        (_, TowerMapObjectKind::SpecialLocation) => -20,
        (_, TowerMapObjectKind::Egg | TowerMapObjectKind::Loot) => -10,
        (_, TowerMapObjectKind::Stairs | TowerMapObjectKind::Exit) => 0,
        (_, TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss) => 30,
        (_, TowerMapObjectKind::Hazard) => 50,
    }
}

fn frontier_bias(goal: TowerRunGoal) -> i32 {
    match goal {
        TowerRunGoal::Scout => -30,
        TowerRunGoal::SafeRun => 10,
        _ => 0,
    }
}

fn first_step(
    map: &TowerMapState,
    start: usize,
    mut target: usize,
    previous: &[Option<usize>],
) -> Option<(i32, i32)> {
    while previous.get(target).copied().flatten()? != start {
        target = previous[target]?;
    }
    let (x, y) = coordinates(map, target);
    Some((
        x as i32 - map.player_x as i32,
        y as i32 - map.player_y as i32,
    ))
}

fn neighbors(map: &TowerMapState, x: u32, y: u32) -> Vec<(u32, u32)> {
    let candidates = [
        (x, y.saturating_sub(1)),
        (x.saturating_sub(1), y),
        (x.saturating_add(1), y),
        (x, y.saturating_add(1)),
    ];
    candidates
        .into_iter()
        .filter(|(next_x, next_y)| *next_x < map.width && *next_y < map.height)
        .collect()
}

fn map_index(map: &TowerMapState, x: u32, y: u32) -> Option<usize> {
    (x < map.width && y < map.height).then_some((y * map.width + x) as usize)
}

fn coordinates(map: &TowerMapState, index: usize) -> (u32, u32) {
    (index as u32 % map.width, index as u32 / map.width)
}
