use crate::data::{GameData, MonsterRole, PassiveSkill, TowerFloorDefinition};
use crate::engine::{monster_engine, town_engine};
use crate::state::{
    DailyCommitment, GameState, ResourceStack, TowerFoundEgg, TowerMapObject, TowerMapObjectKind,
    TowerMapRng, TowerMapState, TowerRoom, TowerRunGoal, TowerRunState, TowerTileKind,
    TowerTileVisibility,
};

pub struct TowerResult {
    pub summary: String,
    pub encounter: Option<TowerEncounterRequest>,
    pub returned_to_town: bool,
}

pub struct TowerEncounterRequest {
    pub floor: u32,
    pub is_boss: bool,
}

pub fn start_run(state: &mut GameState, data: &GameData, goal: TowerRunGoal) -> TowerResult {
    if state.tower_run.is_some() {
        return result("The party is already inside the tower.");
    }

    let ready_members = available_party_ids(state);
    if ready_members.is_empty() {
        return result(
            "Assign at least one rested, uncommitted monster to the party before entering the tower.",
        );
    }

    let start_floor = state
        .tower_progress
        .unlocked_floor
        .max(1)
        .min(max_floor(data));
    let Some(floor) = data.tower_floor(start_floor) else {
        return result(format!("Missing tower floor data for floor {start_floor}."));
    };

    for monster_id in ready_members {
        monster_engine::mark_commitment(state, monster_id, DailyCommitment::Tower);
    }

    let seed = tower_seed(state, start_floor, goal, 0);
    let map = generate_map(state, data, start_floor, goal, seed);
    state.tower_run =
        Some(TowerRunState::new(start_floor, floor.pressure_limit, goal).with_map(map));
    let summary = format!(
        "The party enters floor {}: {}. Move through the map to find stairs, eggs, caches, and enemies.",
        floor.floor, floor.name
    );
    state.activity_log.add(state.day, summary.clone());

    result(summary)
}

pub fn ensure_map(state: &mut GameState, data: &GameData) {
    let Some(run) = &state.tower_run else {
        return;
    };
    let needs_map = run.map.is_empty();

    if needs_map {
        let floor = run.current_floor.max(1).min(max_floor(data));
        let goal = run.goal;
        let seed = tower_seed(state, floor, goal, run.rooms_explored);
        let map = generate_map(state, data, floor, goal, seed);
        if let Some(run) = &mut state.tower_run {
            run.current_floor = floor;
            run.map = map;
            run.add_event(format!("Generated a map for floor {floor}."));
        }
        return;
    }

    if let Some(run) = &mut state.tower_run {
        let restored_visibility = run.map.ensure_visibility();
        if restored_visibility || !run.map.is_visible(run.map.player_x, run.map.player_y) {
            reveal_current_area(&mut run.map);
        }
        if restored_visibility {
            run.add_event("Recovered the party's map notes.".to_owned());
        }
    }
}

pub fn move_party(state: &mut GameState, data: &GameData, dx: i32, dy: i32) -> TowerResult {
    if state.tower_run.is_none() {
        return result("No tower run is active.");
    }
    if dx == 0 && dy == 0 {
        return result("The party waits and listens.");
    }

    ensure_map(state, data);

    let object = {
        let Some(run) = &mut state.tower_run else {
            return result("No tower run is active.");
        };
        if run.map.is_empty() {
            return result("No dungeon map is available.");
        }

        let next_x = run.map.player_x as i32 + dx;
        let next_y = run.map.player_y as i32 + dy;
        if next_x < 0 || next_y < 0 || !run.map.is_passable(next_x as u32, next_y as u32) {
            return result("A wall blocks the way.");
        }

        run.map.player_x = next_x as u32;
        run.map.player_y = next_y as u32;
        run.rooms_explored += 1;
        reveal_current_area(&mut run.map);
        run.map
            .object_index_at(run.map.player_x, run.map.player_y)
            .map(|index| run.map.objects.remove(index))
    };

    let Some(object) = object else {
        return result("The party advances through the dungeon.");
    };

    resolve_map_object(state, data, object)
}

pub fn return_to_town(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(run) = state.tower_run.take() else {
        return result("No tower run is active.");
    };

    record_floor_reached(state, data, run.current_floor);

    for stack in &run.cargo {
        state.resources.add(&stack.resource_id, stack.amount);
    }

    let egg_capacity = town_engine::egg_capacity(state);
    let available_egg_slots = egg_capacity.saturating_sub(state.egg_inventory.eggs.len());
    let mut egg_count = 0;
    for found_egg in run.found_eggs.iter().take(available_egg_slots) {
        state.egg_inventory.add_egg(
            found_egg.egg_type_id.clone(),
            found_egg.hatch_days,
            found_egg.origin_floor,
            found_egg.palette_seed,
        );
        egg_count += 1;
    }
    let eggs_left_behind = run.found_eggs.len().saturating_sub(egg_count);

    let cargo_label = cargo_text(data, &run.cargo);
    let mut summary = format!(
        "Returned from floor {} with {} and {} egg(s).",
        run.current_floor, cargo_label, egg_count
    );
    if eggs_left_behind > 0 {
        summary.push_str(&format!(
            " Hatchery capacity left {} egg(s) behind ({}/{}).",
            eggs_left_behind,
            state.egg_inventory.eggs.len(),
            egg_capacity
        ));
    }
    state.activity_log.add(state.day, summary.clone());

    TowerResult {
        summary,
        encounter: None,
        returned_to_town: true,
    }
}

pub fn cargo_text(data: &GameData, cargo: &[ResourceStack]) -> String {
    if cargo.is_empty() {
        return "no materials".to_owned();
    }

    cargo
        .iter()
        .filter(|stack| stack.amount > 0)
        .map(|stack| {
            format!(
                "{} {}",
                stack.amount,
                data.resource_name(&stack.resource_id)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn party_count(state: &GameState) -> usize {
    state
        .monster_roster
        .party_slots
        .iter()
        .filter(|slot| slot.is_some())
        .count()
}

pub fn battle_ready_party_count(state: &GameState) -> usize {
    state
        .monster_roster
        .party_slots
        .iter()
        .filter_map(|slot| state.monster_roster.monster((*slot)?))
        .filter(|monster| monster.is_battle_ready() && state.town.monster_job(monster.id).is_none())
        .count()
}

fn resolve_map_object(
    state: &mut GameState,
    data: &GameData,
    object: TowerMapObject,
) -> TowerResult {
    match object.kind {
        TowerMapObjectKind::Loot => {
            let resource_name = data.resource_name(&object.resource_id).to_owned();
            if let Some(run) = &mut state.tower_run {
                run.add_cargo(&object.resource_id, object.amount);
                let summary = format!(
                    "Found {} {} in a tower cache.",
                    object.amount, resource_name
                );
                run.add_event(summary.clone());
                result(summary)
            } else {
                result("No tower run is active.")
            }
        }
        TowerMapObjectKind::Egg => {
            let egg_name = data
                .egg_type(&object.egg_type_id)
                .map(|egg| egg.name.as_str())
                .unwrap_or(object.egg_type_id.as_str())
                .to_owned();
            if let Some(run) = &mut state.tower_run {
                run.found_eggs.push(TowerFoundEgg {
                    egg_type_id: object.egg_type_id,
                    hatch_days: object.hatch_days,
                    origin_floor: run.current_floor,
                    palette_seed: object.palette_seed,
                });
                let summary = format!("Found a {egg_name} in a quiet nest.");
                run.add_event(summary.clone());
                result(summary)
            } else {
                result("No tower run is active.")
            }
        }
        TowerMapObjectKind::Enemy | TowerMapObjectKind::Boss => {
            let is_boss = object.kind == TowerMapObjectKind::Boss;
            let floor = state
                .tower_run
                .as_ref()
                .map(|run| run.current_floor)
                .unwrap_or(1);
            let label = if is_boss { "boss" } else { "enemy" };
            let summary = format!("A {label} blocks the tile. Combat starts.");
            if let Some(run) = &mut state.tower_run {
                run.add_event(summary.clone());
            }
            TowerResult {
                summary,
                encounter: Some(TowerEncounterRequest { floor, is_boss }),
                returned_to_town: false,
            }
        }
        TowerMapObjectKind::Stairs => advance_floor(state, data),
        TowerMapObjectKind::Exit => return_to_town(state, data),
    }
}

fn advance_floor(state: &mut GameState, data: &GameData) -> TowerResult {
    let Some(run) = &state.tower_run else {
        return result("No tower run is active.");
    };
    let next_floor = (run.current_floor + 1).min(max_floor(data));
    if next_floor == run.current_floor {
        return result("The stairs end at the tower crown.");
    }

    let Some(next_floor_data) = data.tower_floor(next_floor) else {
        return result(format!("Missing tower floor data for floor {next_floor}."));
    };

    record_floor_reached(state, data, next_floor);
    let goal = state
        .tower_run
        .as_ref()
        .map(|run| run.goal)
        .unwrap_or_default();
    let step_count = state
        .tower_run
        .as_ref()
        .map(|run| run.rooms_explored)
        .unwrap_or(0);
    let seed = tower_seed(state, next_floor, goal, step_count);
    let map = generate_map(state, data, next_floor, goal, seed);

    if let Some(run) = &mut state.tower_run {
        run.current_floor = next_floor;
        run.pressure_limit = next_floor_data.pressure_limit;
        run.map = map;
        let summary = format!(
            "Descended to floor {}: {}. A fresh map unfolds.",
            next_floor_data.floor, next_floor_data.name
        );
        run.add_event(summary.clone());
        return result(summary);
    }

    result("No tower run is active.")
}

fn generate_map(
    state: &GameState,
    data: &GameData,
    floor_number: u32,
    goal: TowerRunGoal,
    seed: u64,
) -> TowerMapState {
    let mut rng = TowerMapRng::new(seed);
    let width = 30 + (floor_number / 4).min(2) * 2;
    let height = 22;
    let mut map = TowerMapState::new(width, height, floor_number, seed);
    let room_target = room_target(floor_number, goal);

    for _ in 0..room_target * 12 {
        if map.rooms.len() >= room_target as usize {
            break;
        }
        let room_width = rng.range(5, 9);
        let room_height = rng.range(4, 7);
        let room = TowerRoom {
            width: room_width,
            height: room_height,
            start_x: rng.range(1, width.saturating_sub(room_width + 1).max(2)),
            start_y: rng.range(1, height.saturating_sub(room_height + 1).max(2)),
        };

        if room.start_x + room.width >= width - 1 || room.start_y + room.height >= height - 1 {
            continue;
        }
        if map
            .rooms
            .iter()
            .any(|existing| room.intersects_padded(*existing))
        {
            continue;
        }

        carve_room(&mut map, room);
        if let Some(previous) = map.rooms.last().copied() {
            carve_corridor(&mut map, previous.center(), room.center(), &mut rng);
        }
        map.rooms.push(room);
    }

    if map.rooms.len() < 4 {
        carve_fallback_layout(&mut map, &mut rng);
    }

    let start_room = map.rooms[0];
    let (start_x, start_y) = start_room.random_inner(&mut rng);
    map.start_x = start_x;
    map.start_y = start_y;
    map.player_x = start_x;
    map.player_y = start_y;

    add_map_objects(&mut map, state, data, floor_number, goal, &mut rng);
    reveal_current_area(&mut map);
    map
}

fn room_target(floor: u32, goal: TowerRunGoal) -> u32 {
    let base = 8 + (floor / 2).min(5);
    match goal {
        TowerRunGoal::Scout => base + 2,
        TowerRunGoal::PushDeeper => base + 1,
        TowerRunGoal::SafeRun => base.saturating_sub(1),
        _ => base,
    }
}

fn carve_room(map: &mut TowerMapState, room: TowerRoom) {
    for x in room.start_x + 1..room.start_x + room.width - 1 {
        for y in room.start_y + 1..room.start_y + room.height - 1 {
            map.set_tile(x, y, TowerTileKind::Floor);
        }
    }
}

fn carve_corridor(
    map: &mut TowerMapState,
    start: (u32, u32),
    end: (u32, u32),
    rng: &mut TowerMapRng,
) {
    if rng.chance(1, 2) {
        carve_horizontal(map, start.0, end.0, start.1);
        carve_vertical(map, start.1, end.1, end.0);
    } else {
        carve_vertical(map, start.1, end.1, start.0);
        carve_horizontal(map, start.0, end.0, end.1);
    }
}

fn carve_fallback_layout(map: &mut TowerMapState, rng: &mut TowerMapRng) {
    map.tiles.fill(TowerTileKind::Wall);
    map.rooms.clear();

    let width = map.width;
    let height = map.height;
    let rooms = [
        TowerRoom {
            start_x: 2,
            start_y: 2,
            width: 7,
            height: 6,
        },
        TowerRoom {
            start_x: width / 2 - 4,
            start_y: 2,
            width: 8,
            height: 6,
        },
        TowerRoom {
            start_x: width - 10,
            start_y: 3,
            width: 8,
            height: 6,
        },
        TowerRoom {
            start_x: 3,
            start_y: height - 9,
            width: 8,
            height: 6,
        },
        TowerRoom {
            start_x: width / 2 - 3,
            start_y: height - 8,
            width: 7,
            height: 6,
        },
        TowerRoom {
            start_x: width - 11,
            start_y: height - 9,
            width: 9,
            height: 6,
        },
    ];

    let mut previous_center = None;
    for room in rooms {
        carve_room(map, room);
        if let Some(previous) = previous_center {
            carve_corridor(map, previous, room.center(), rng);
        }
        previous_center = Some(room.center());
        map.rooms.push(room);
    }
}

fn carve_horizontal(map: &mut TowerMapState, from_x: u32, to_x: u32, y: u32) {
    let start = from_x.min(to_x);
    let end = from_x.max(to_x);
    for x in start..=end {
        carve_corridor_tile(map, x, y);
    }
}

fn carve_vertical(map: &mut TowerMapState, from_y: u32, to_y: u32, x: u32) {
    let start = from_y.min(to_y);
    let end = from_y.max(to_y);
    for y in start..=end {
        carve_corridor_tile(map, x, y);
    }
}

fn carve_corridor_tile(map: &mut TowerMapState, x: u32, y: u32) {
    if map.tile_at(x, y) == TowerTileKind::Wall {
        map.set_tile(x, y, TowerTileKind::Corridor);
    }
}

fn reveal_current_area(map: &mut TowerMapState) {
    map.ensure_visibility();
    for visibility in &mut map.visibility {
        if *visibility == TowerTileVisibility::Visible {
            *visibility = TowerTileVisibility::Explored;
        }
    }

    let player_x = map.player_x;
    let player_y = map.player_y;
    if let Some(room) = room_containing(map, player_x, player_y) {
        reveal_room(map, room);
        reveal_radius(map, player_x, player_y, 2);
        reveal_room_edges(map, room);
    } else {
        reveal_radius(map, player_x, player_y, 3);
    }
}

fn room_containing(map: &TowerMapState, x: u32, y: u32) -> Option<TowerRoom> {
    map.rooms.iter().copied().find(|room| {
        x >= room.start_x
            && x < room.start_x + room.width
            && y >= room.start_y
            && y < room.start_y + room.height
    })
}

fn reveal_room(map: &mut TowerMapState, room: TowerRoom) {
    let max_x = (room.start_x + room.width).min(map.width);
    let max_y = (room.start_y + room.height).min(map.height);
    for y in room.start_y..max_y {
        for x in room.start_x..max_x {
            map.set_visibility(x, y, TowerTileVisibility::Visible);
        }
    }
}

fn reveal_room_edges(map: &mut TowerMapState, room: TowerRoom) {
    let min_x = room.start_x.saturating_sub(1);
    let min_y = room.start_y.saturating_sub(1);
    let max_x = (room.start_x + room.width).min(map.width.saturating_sub(1));
    let max_y = (room.start_y + room.height).min(map.height.saturating_sub(1));

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if map.is_passable(x, y) {
                map.set_visibility(x, y, TowerTileVisibility::Visible);
            }
        }
    }
}

fn reveal_radius(map: &mut TowerMapState, center_x: u32, center_y: u32, radius: u32) {
    let min_x = center_x.saturating_sub(radius);
    let min_y = center_y.saturating_sub(radius);
    let max_x = (center_x + radius).min(map.width.saturating_sub(1));
    let max_y = (center_y + radius).min(map.height.saturating_sub(1));

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let distance = center_x.abs_diff(x) + center_y.abs_diff(y);
            if distance <= radius + 1 {
                map.set_visibility(x, y, TowerTileVisibility::Visible);
            }
        }
    }
}

fn add_map_objects(
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

    if floor.is_boss_floor {
        place_object(map, TowerMapObject::boss(0, 0), rng);
    } else {
        for _ in 0..enemy_count(map, state, floor_number, goal) {
            place_object(map, TowerMapObject::enemy(0, 0), rng);
        }
    }
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

impl TowerMapObject {
    fn stairs(x: u32, y: u32) -> Self {
        Self {
            kind: TowerMapObjectKind::Stairs,
            x,
            y,
            resource_id: String::new(),
            amount: 0,
            egg_type_id: String::new(),
            hatch_days: 0,
            palette_seed: 0,
        }
    }

    fn exit(x: u32, y: u32) -> Self {
        Self {
            kind: TowerMapObjectKind::Exit,
            x,
            y,
            resource_id: String::new(),
            amount: 0,
            egg_type_id: String::new(),
            hatch_days: 0,
            palette_seed: 0,
        }
    }

    fn enemy(x: u32, y: u32) -> Self {
        Self {
            kind: TowerMapObjectKind::Enemy,
            x,
            y,
            resource_id: String::new(),
            amount: 0,
            egg_type_id: String::new(),
            hatch_days: 0,
            palette_seed: 0,
        }
    }

    fn boss(x: u32, y: u32) -> Self {
        Self {
            kind: TowerMapObjectKind::Boss,
            x,
            y,
            resource_id: String::new(),
            amount: 0,
            egg_type_id: String::new(),
            hatch_days: 0,
            palette_seed: 0,
        }
    }
}

fn available_party_ids(state: &GameState) -> Vec<u64> {
    state
        .monster_roster
        .party_slots
        .iter()
        .filter_map(|slot| {
            let monster = state.monster_roster.monster((*slot)?)?;
            if monster_engine::can_take_daily_action(state, monster).is_ok() {
                Some(monster.id)
            } else {
                None
            }
        })
        .collect()
}

fn record_floor_reached(state: &mut GameState, data: &GameData, floor: u32) {
    state.tower_progress.best_floor = state.tower_progress.best_floor.max(floor);

    if let Some(floor_data) = data.tower_floor(floor) {
        let unlocked = floor_data.unlocks_floor.max(floor);
        state.tower_progress.unlocked_floor = state
            .tower_progress
            .unlocked_floor
            .max(unlocked)
            .min(max_floor(data));
    }
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

fn tower_seed(state: &GameState, floor: u32, goal: TowerRunGoal, salt: u32) -> u64 {
    0x544F_5745_524D_4150
        ^ u64::from(state.day).wrapping_mul(97)
        ^ u64::from(floor).wrapping_mul(193)
        ^ u64::from(salt).wrapping_mul(389)
        ^ state.egg_inventory.next_id.wrapping_mul(53)
        ^ (goal as u64).wrapping_mul(577)
}

fn max_floor(data: &GameData) -> u32 {
    data.tower_floors
        .iter()
        .map(|floor| floor.floor)
        .max()
        .unwrap_or(1)
}

fn result(summary: impl Into<String>) -> TowerResult {
    TowerResult {
        summary: summary.into(),
        encounter: None,
        returned_to_town: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameDataLoader;

    #[test]
    fn generated_map_has_start_and_stairs() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let state = GameState::new(&data);
        let map = generate_map(&state, &data, 1, TowerRunGoal::Scout, 42);

        assert!(map.is_passable(map.player_x, map.player_y));
        assert!(map
            .objects
            .iter()
            .any(|object| object.kind == TowerMapObjectKind::Stairs));
        assert!(map.rooms.len() >= 4);
        assert!(map.is_visible(map.player_x, map.player_y));
        assert!(map.visibility.contains(&TowerTileVisibility::Hidden));
    }

    #[test]
    fn movement_collects_object_on_destination_tile() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);

        start_run(&mut state, &data, TowerRunGoal::Salvage);
        let run = state.tower_run.as_ref().expect("tower run should start");
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (dx, dy, target_x, target_y) = directions
            .iter()
            .find_map(|(dx, dy)| {
                let x = run.map.player_x as i32 + dx;
                let y = run.map.player_y as i32 + dy;
                if x >= 0 && y >= 0 && run.map.is_passable(x as u32, y as u32) {
                    Some((*dx, *dy, x as u32, y as u32))
                } else {
                    None
                }
            })
            .expect("start room should have an adjacent passable tile");

        let run = state.tower_run.as_mut().expect("tower run should exist");
        run.map
            .objects
            .retain(|object| object.x != target_x || object.y != target_y);
        run.map.objects.push(TowerMapObject {
            kind: TowerMapObjectKind::Loot,
            x: target_x,
            y: target_y,
            resource_id: "wood".to_owned(),
            amount: 3,
            egg_type_id: String::new(),
            hatch_days: 0,
            palette_seed: 0,
        });

        let result = move_party(&mut state, &data, dx, dy);
        let run = state.tower_run.as_ref().expect("tower run should remain");

        assert!(result.summary.contains("Found 3"));
        assert_eq!((run.map.player_x, run.map.player_y), (target_x, target_y));
        assert_eq!(run.rooms_explored, 1);
        assert!(run.map.object_at(target_x, target_y).is_none());
        assert_eq!(run.cargo_amount(), 3);
    }

    #[test]
    fn movement_keeps_only_discovered_tiles_revealed() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);

        start_run(&mut state, &data, TowerRunGoal::Scout);
        let run = state.tower_run.as_ref().expect("tower run should start");
        let hidden_before = run
            .map
            .visibility
            .iter()
            .filter(|visibility| **visibility == TowerTileVisibility::Hidden)
            .count();
        let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .find_map(|(dx, dy)| {
                let x = run.map.player_x as i32 + dx;
                let y = run.map.player_y as i32 + dy;
                if x >= 0 && y >= 0 && run.map.is_passable(x as u32, y as u32) {
                    Some((*dx, *dy))
                } else {
                    None
                }
            })
            .expect("start room should have an adjacent passable tile");

        move_party(&mut state, &data, dx, dy);
        let run = state.tower_run.as_ref().expect("tower run should remain");
        let hidden_after = run
            .map
            .visibility
            .iter()
            .filter(|visibility| **visibility == TowerTileVisibility::Hidden)
            .count();

        assert!(run.map.is_visible(run.map.player_x, run.map.player_y));
        assert!(hidden_after > 0);
        assert!(hidden_after <= hidden_before);
    }

    #[test]
    fn reveal_current_area_marks_previous_tiles_explored() {
        let mut map = TowerMapState::new(12, 5, 1, 7);
        for x in 1..11 {
            map.set_tile(x, 2, TowerTileKind::Corridor);
        }
        map.player_x = 1;
        map.player_y = 2;
        reveal_current_area(&mut map);

        assert!(map.is_visible(1, 2));

        map.player_x = 10;
        map.player_y = 2;
        reveal_current_area(&mut map);

        assert_eq!(map.visibility_at(1, 2), TowerTileVisibility::Explored);
        assert!(map.is_visible(10, 2));
    }

    #[test]
    fn ensure_map_restores_visibility_for_older_runs() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);

        start_run(&mut state, &data, TowerRunGoal::SafeRun);
        {
            let run = state.tower_run.as_mut().expect("tower run should start");
            run.map.visibility.clear();
        }

        ensure_map(&mut state, &data);
        let run = state.tower_run.as_ref().expect("tower run should remain");

        assert_eq!(
            run.map.visibility.len(),
            (run.map.width * run.map.height) as usize
        );
        assert!(run.map.is_visible(run.map.player_x, run.map.player_y));
        assert!(run
            .event_log
            .iter()
            .any(|event| event.contains("map notes")));
    }

    #[test]
    fn return_to_town_respects_hatchery_egg_capacity() {
        let data = GameDataLoader::load_embedded().expect("embedded data should load");
        let mut state = GameState::new(&data);
        state.town.set_building_level("hatchery", 1);
        state
            .egg_inventory
            .add_egg("mossy_egg".to_owned(), 1, 1, 0x101);
        state
            .egg_inventory
            .add_egg("mossy_egg".to_owned(), 1, 1, 0x102);

        let mut run = TowerRunState::new(1, 9, TowerRunGoal::EggHunt);
        run.found_eggs.push(TowerFoundEgg {
            egg_type_id: "mossy_egg".to_owned(),
            hatch_days: 1,
            origin_floor: 1,
            palette_seed: 0x201,
        });
        run.found_eggs.push(TowerFoundEgg {
            egg_type_id: "mossy_egg".to_owned(),
            hatch_days: 1,
            origin_floor: 1,
            palette_seed: 0x202,
        });
        state.tower_run = Some(run);

        let result = return_to_town(&mut state, &data);

        assert_eq!(state.egg_inventory.eggs.len(), 3);
        assert!(result.summary.contains("1 egg(s)"));
        assert!(result.summary.contains("left 1 egg(s) behind"));
    }
}
