use crate::data::PassiveSkill;
use crate::state::{GameState, TowerMapObjectKind, TowerMapState};

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

#[cfg(test)]
mod tests;
