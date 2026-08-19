use crate::data::PassiveSkill;
use crate::state::{GameState, TowerMapObjectKind, TowerMapState, TowerRoom, TowerTileVisibility};

pub(super) fn party_has_passive(state: &GameState, passive: PassiveSkill) -> bool {
    state
        .monster_roster
        .party_slots
        .iter()
        .flatten()
        .filter_map(|id| state.monster_roster.monster(*id))
        .any(|monster| monster.passive == passive)
}

pub(super) fn reveal_secret_in_current_room(map: &mut TowerMapState) -> usize {
    let Some(room) = map.rooms.iter().find(|room| {
        map.player_x >= room.start_x
            && map.player_x < room.start_x + room.width
            && map.player_y >= room.start_y
            && map.player_y < room.start_y + room.height
    }) else {
        return 0;
    };
    let max_x = room.start_x + room.width;
    let max_y = room.start_y + room.height;
    let mut revealed = 0;
    for object in &mut map.objects {
        if object.kind == TowerMapObjectKind::SecretCache
            && !object.revealed
            && object.x >= room.start_x
            && object.x < max_x
            && object.y >= room.start_y
            && object.y < max_y
        {
            object.revealed = true;
            revealed += 1;
        }
    }
    revealed
}

pub(super) fn chart_nearest_secret_rooms(map: &mut TowerMapState, count: u32) -> usize {
    let mut secrets = map
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| object.kind == TowerMapObjectKind::SecretCache && !object.revealed)
        .map(|(index, object)| {
            (
                map.player_x.abs_diff(object.x) + map.player_y.abs_diff(object.y),
                index,
                object.x,
                object.y,
            )
        })
        .collect::<Vec<_>>();
    secrets.sort_by_key(|entry| (entry.0, entry.1));

    let selected = secrets.into_iter().take(count as usize).collect::<Vec<_>>();
    for (_, index, x, y) in &selected {
        map.objects[*index].revealed = true;
        if let Some(room) = map
            .rooms
            .iter()
            .copied()
            .find(|room| point_in_room(*room, *x, *y))
        {
            reveal_room(map, room);
        }
    }
    selected.len()
}

fn point_in_room(room: TowerRoom, x: u32, y: u32) -> bool {
    x >= room.start_x
        && x < room.start_x + room.width
        && y >= room.start_y
        && y < room.start_y + room.height
}

fn reveal_room(map: &mut TowerMapState, room: TowerRoom) {
    map.ensure_visibility();
    let max_x = (room.start_x + room.width).min(map.width);
    let max_y = (room.start_y + room.height).min(map.height);
    for y in room.start_y..max_y {
        for x in room.start_x..max_x {
            if map.is_passable(x, y) {
                map.set_visibility(x, y, TowerTileVisibility::Visible);
            }
        }
    }
}

#[cfg(test)]
mod tests;
