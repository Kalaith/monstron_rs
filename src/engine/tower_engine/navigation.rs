use std::collections::VecDeque;

use crate::state::{TowerMapState, TowerTileVisibility};

pub(super) fn explore_direction(map: &TowerMapState) -> Option<(i32, i32)> {
    if map.is_empty() {
        return None;
    }

    let start = map_index(map, map.player_x, map.player_y)?;
    let mut queue = VecDeque::from([start]);
    let mut visited = vec![false; (map.width * map.height) as usize];
    let mut previous = vec![None; visited.len()];
    visited[start] = true;

    while let Some(index) = queue.pop_front() {
        let (x, y) = coordinates(map, index);
        if index != start && is_explore_target(map, x, y) {
            return first_step(map, start, index, &previous);
        }

        for (next_x, next_y) in neighbors(map, x, y) {
            let Some(next) = map_index(map, next_x, next_y) else {
                continue;
            };
            if visited[next] || !map.is_passable(next_x, next_y) {
                continue;
            }
            visited[next] = true;
            previous[next] = Some(index);
            queue.push_back(next);
        }
    }
    None
}

fn is_explore_target(map: &TowerMapState, x: u32, y: u32) -> bool {
    (map.is_discovered(x, y) && map.object_at(x, y).is_some())
        || map.visibility_at(x, y) == TowerTileVisibility::Hidden
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
