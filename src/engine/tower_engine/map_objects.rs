use super::max_floor;
use crate::data::{GameData, MonsterRole, PassiveSkill, TowerFloorDefinition};
use crate::state::{
    GameState, TowerMapObject, TowerMapObjectKind, TowerMapRng, TowerMapState, TowerRunGoal,
};

pub(super) fn add_map_objects(
    map: &mut TowerMapState,
    state: &GameState,
    data: &GameData,
    floor_number: u32,
    goal: TowerRunGoal,
    rng: &mut TowerMapRng,
) {
    let Some(floor) = data.tower_floor(floor_number) else {
        return;
    };

    if floor.floor < max_floor(data) && !floor.is_boss_floor {
        place_object(map, TowerMapObject::stairs(0, 0), rng);
    }
    if floor.floor % 5 == 0 || floor.is_boss_floor {
        place_object(map, TowerMapObject::exit(0, 0), rng);
    }

    for object in loot_objects(floor, goal, rng) {
        place_object(map, object, rng);
    }
    for object in egg_objects(floor, data, goal, rng) {
        place_object(map, object, rng);
    }
    for object in special_location_objects(data, floor_number, goal, rng) {
        place_room_landmark(map, object, rng);
    }
    for object in hazard_objects(data, floor_number, goal, rng) {
        place_object(map, object, rng);
    }

    if floor.is_boss_floor {
        if let Some(enemy) = eligible_enemies(data, floor_number, true).first() {
            place_object(map, TowerMapObject::boss(&enemy.id), rng);
        }
    } else {
        let enemies = eligible_enemies(data, floor_number, false);
        for _ in 0..enemy_count(map, state, floor_number, goal) {
            if let Some(enemy) = enemies.get(rng.range(0, enemies.len() as u32) as usize) {
                place_object(map, TowerMapObject::enemy(&enemy.id), rng);
            }
        }
    }
}

fn eligible_enemies(
    data: &GameData,
    floor_number: u32,
    is_boss: bool,
) -> Vec<&crate::data::EnemyDefinition> {
    data.enemies
        .iter()
        .filter(|enemy| {
            enemy.is_boss == is_boss
                && enemy.min_floor <= floor_number
                && enemy.max_floor >= floor_number
        })
        .collect()
}

fn special_location_objects(
    data: &GameData,
    floor_number: u32,
    goal: TowerRunGoal,
    rng: &mut TowerMapRng,
) -> Vec<TowerMapObject> {
    let eligible: Vec<_> = data
        .tower_special_locations
        .iter()
        .filter(|location| location.min_floor <= floor_number && location.max_floor >= floor_number)
        .collect();
    if eligible.is_empty() {
        return Vec::new();
    }

    let count = if goal == TowerRunGoal::Scout || (eligible.len() > 1 && rng.chance(1, 3)) {
        2
    } else {
        1
    };
    let start = rng.range(0, eligible.len() as u32) as usize;
    (0..count.min(eligible.len()))
        .filter_map(|offset| {
            let location = eligible[(start + offset) % eligible.len()];
            let event_id = location
                .event_ids
                .get(rng.range(0, location.event_ids.len() as u32) as usize)?;
            Some(TowerMapObject::special_location(&location.id, event_id))
        })
        .collect()
}

fn hazard_objects(
    data: &GameData,
    floor_number: u32,
    goal: TowerRunGoal,
    rng: &mut TowerMapRng,
) -> Vec<TowerMapObject> {
    if goal == TowerRunGoal::SafeRun {
        return Vec::new();
    }
    let eligible: Vec<_> = data
        .tower_hazards
        .iter()
        .filter(|hazard| hazard.min_floor <= floor_number && hazard.max_floor >= floor_number)
        .collect();
    if eligible.is_empty() {
        return Vec::new();
    }
    let count = if goal == TowerRunGoal::PushDeeper || floor_number >= 8 {
        2
    } else {
        1
    };
    (0..count)
        .filter_map(|_| {
            let hazard = eligible.get(rng.range(0, eligible.len() as u32) as usize)?;
            Some(TowerMapObject::hazard(&hazard.id))
        })
        .collect()
}

fn loot_objects(
    floor: &TowerFloorDefinition,
    goal: TowerRunGoal,
    rng: &mut TowerMapRng,
) -> Vec<TowerMapObject> {
    if floor.loot.is_empty() {
        return Vec::new();
    }

    let mut count = 3 + (floor.floor / 3).min(3);
    match goal {
        TowerRunGoal::Salvage => count += 3,
        TowerRunGoal::EggHunt | TowerRunGoal::SafeRun => count = count.saturating_sub(1),
        _ => {}
    }

    (0..count)
        .filter_map(|_| {
            let base = floor
                .loot
                .get(rng.range(0, floor.loot.len() as u32) as usize)?;
            let goal_bonus = if goal == TowerRunGoal::Salvage { 2 } else { 0 };
            Some(TowerMapObject {
                kind: TowerMapObjectKind::Loot,
                x: 0,
                y: 0,
                resource_id: base.resource_id.clone(),
                amount: base.amount + goal_bonus + rng.range(0, 3) as i32,
                egg_type_id: String::new(),
                hatch_days: 0,
                palette_seed: 0,
                enemy_id: String::new(),
                special_location_id: String::new(),
                event_id: String::new(),
                hazard_id: String::new(),
            })
        })
        .collect()
}

fn egg_objects(
    floor: &TowerFloorDefinition,
    data: &GameData,
    goal: TowerRunGoal,
    rng: &mut TowerMapRng,
) -> Vec<TowerMapObject> {
    if floor.egg_types.is_empty() {
        return Vec::new();
    }

    let count = match goal {
        TowerRunGoal::EggHunt => 2 + rng.range(0, 2),
        TowerRunGoal::Salvage => rng.range(0, 2),
        _ if rng.chance(2, 3) => 1,
        _ => 0,
    };

    (0..count)
        .filter_map(|_| {
            let egg_id = floor
                .egg_types
                .get(rng.range(0, floor.egg_types.len() as u32) as usize)?;
            let egg = data.egg_type(egg_id)?;
            Some(TowerMapObject {
                kind: TowerMapObjectKind::Egg,
                x: 0,
                y: 0,
                resource_id: String::new(),
                amount: 0,
                egg_type_id: egg.id.clone(),
                hatch_days: egg.hatch_days,
                palette_seed: 0xE66_0000 ^ u64::from(floor.floor) << 24 ^ u64::from(rng.next_u32()),
                enemy_id: String::new(),
                special_location_id: String::new(),
                event_id: String::new(),
                hazard_id: String::new(),
            })
        })
        .collect()
}

fn enemy_count(
    map: &TowerMapState,
    state: &GameState,
    floor_number: u32,
    goal: TowerRunGoal,
) -> u32 {
    let profile = party_profile(state);
    let mut count = (map.rooms.len() as u32 / 2 + floor_number / 3).clamp(2, 9);
    match goal {
        TowerRunGoal::PushDeeper => count += 2,
        TowerRunGoal::SafeRun | TowerRunGoal::Scout => count = count.saturating_sub(2).max(1),
        _ => {}
    }
    if profile.safety >= 3 {
        count = count.saturating_sub(1).max(1);
    }
    count
}

#[derive(Default)]
struct TowerPartyProfile {
    safety: u32,
}

fn party_profile(state: &GameState) -> TowerPartyProfile {
    let mut profile = TowerPartyProfile::default();
    for monster_id in state.monster_roster.party_slots.iter().flatten() {
        let Some(monster) = state.monster_roster.monster(*monster_id) else {
            continue;
        };
        match monster.role {
            MonsterRole::Tank | MonsterRole::Support => profile.safety += 1,
            MonsterRole::Scout | MonsterRole::Striker => {}
        }
        match monster.passive {
            PassiveSkill::ResistsPoison | PassiveSkill::SoothesInjuries => profile.safety += 2,
            PassiveSkill::FindsSmallLoot
            | PassiveSkill::DetectsEggs
            | PassiveSkill::FindsStone
            | PassiveSkill::BurnsBrambles => {}
        }
    }
    profile
}

fn place_object(map: &mut TowerMapState, mut object: TowerMapObject, rng: &mut TowerMapRng) {
    if map.rooms.is_empty() {
        return;
    }

    for _ in 0..80 {
        let room_index = if map.rooms.len() > 1 {
            rng.range(1, map.rooms.len() as u32) as usize
        } else {
            0
        };
        let (x, y) = map.rooms[room_index].random_inner(rng);
        if x == map.player_x && y == map.player_y {
            continue;
        }
        if !map.is_passable(x, y) || map.object_at(x, y).is_some() {
            continue;
        }

        object.x = x;
        object.y = y;
        map.objects.push(object);
        return;
    }
}

pub(super) fn spawn_pressure_enemy(
    map: &mut TowerMapState,
    data: &GameData,
    seed: u64,
) -> Option<String> {
    let eligible = eligible_enemies(data, map.floor, false);
    if eligible.is_empty() {
        return None;
    }
    let mut rng = TowerMapRng::new(seed);
    let enemy = eligible[rng.range(0, eligible.len() as u32) as usize];
    let before = map.objects.len();
    place_object(map, TowerMapObject::enemy(&enemy.id), &mut rng);
    (map.objects.len() > before).then(|| enemy.id.clone())
}

fn place_room_landmark(map: &mut TowerMapState, mut object: TowerMapObject, rng: &mut TowerMapRng) {
    if map.rooms.len() <= 1 {
        return;
    }
    for _ in 0..80 {
        let room = map.rooms[rng.range(1, map.rooms.len() as u32) as usize];
        if map.objects.iter().any(|existing| {
            existing.x >= room.start_x
                && existing.x < room.start_x + room.width
                && existing.y >= room.start_y
                && existing.y < room.start_y + room.height
        }) {
            continue;
        }
        let (x, y) = room.center();
        object.x = x;
        object.y = y;
        map.objects.push(object);
        return;
    }
}

impl TowerMapObject {
    fn empty(kind: TowerMapObjectKind) -> Self {
        Self {
            kind,
            x: 0,
            y: 0,
            resource_id: String::new(),
            amount: 0,
            egg_type_id: String::new(),
            hatch_days: 0,
            palette_seed: 0,
            enemy_id: String::new(),
            special_location_id: String::new(),
            event_id: String::new(),
            hazard_id: String::new(),
        }
    }

    fn stairs(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            ..Self::empty(TowerMapObjectKind::Stairs)
        }
    }

    fn exit(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            ..Self::empty(TowerMapObjectKind::Exit)
        }
    }

    fn enemy(enemy_id: &str) -> Self {
        Self {
            enemy_id: enemy_id.to_owned(),
            ..Self::empty(TowerMapObjectKind::Enemy)
        }
    }

    fn boss(enemy_id: &str) -> Self {
        Self {
            enemy_id: enemy_id.to_owned(),
            ..Self::empty(TowerMapObjectKind::Boss)
        }
    }

    fn special_location(location_id: &str, event_id: &str) -> Self {
        Self {
            special_location_id: location_id.to_owned(),
            event_id: event_id.to_owned(),
            ..Self::empty(TowerMapObjectKind::SpecialLocation)
        }
    }

    fn hazard(hazard_id: &str) -> Self {
        Self {
            hazard_id: hazard_id.to_owned(),
            ..Self::empty(TowerMapObjectKind::Hazard)
        }
    }
}
