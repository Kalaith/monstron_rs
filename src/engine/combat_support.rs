use crate::data::{EnemyDefinition, GameData, MonsterRole};
use crate::state::{
    CombatOutcome, CombatSide, CombatState, CombatTurn, Combatant, DailyCommitment, GameState,
    ResourceStack, TowerFoundEgg, TowerRunState,
};

pub(crate) fn build_allies(state: &GameState) -> Vec<Combatant> {
    state
        .monster_roster
        .party_slots
        .iter()
        .enumerate()
        .filter_map(|(slot, monster_id)| {
            let monster = state.monster_roster.monster((*monster_id)?)?;
            if !monster.is_battle_ready() {
                return None;
            }
            if state.town.monster_job(monster.id).is_some() {
                return None;
            }
            if state.tower_run.is_some() && monster.condition.commitment != DailyCommitment::Tower {
                return None;
            }
            let penalty = monster.fatigue_penalty();
            let max_hp = (monster.max_hp - penalty * 2).max(1);
            Some(Combatant {
                name: monster.name.clone(),
                source_id: monster.species_id.clone(),
                monster_id: Some(monster.id),
                role: Some(monster.role),
                slot,
                level: monster.level,
                max_hp,
                hp: monster.hp.min(max_hp),
                attack: (monster.attack - penalty).max(1),
                defense: (monster.defense - penalty).max(0),
                speed: (monster.speed - penalty).max(1),
                morale: 50 + monster.bond as i32 - monster.condition.fatigue as i32 * 4,
                is_defending: false,
                is_guarding: false,
                is_marked: false,
                visual_seed: monster.visual_seed,
            })
        })
        .collect()
}

pub(crate) fn build_enemies(data: &GameData, floor: u32, is_boss: bool) -> Vec<Combatant> {
    let eligible: Vec<&EnemyDefinition> = data
        .enemies
        .iter()
        .filter(|enemy| {
            enemy.is_boss == is_boss && enemy.min_floor <= floor && enemy.max_floor >= floor
        })
        .collect();
    let count = if is_boss { 1 } else { (1 + floor / 4).min(3) };

    (0..count as usize)
        .filter_map(|slot| {
            let enemy = eligible.get((floor as usize + slot) % eligible.len().max(1))?;
            Some(Combatant {
                name: enemy.name.clone(),
                source_id: enemy.id.clone(),
                monster_id: None,
                role: None,
                slot,
                level: floor,
                max_hp: enemy.max_hp + floor as i32,
                hp: enemy.max_hp + floor as i32,
                attack: enemy.attack + floor as i32 / 3,
                defense: enemy.defense,
                speed: enemy.speed,
                morale: if is_boss { 90 } else { 35 + floor as i32 * 3 },
                is_defending: false,
                is_guarding: false,
                is_marked: false,
                visual_seed: hash_id(&enemy.id) ^ u64::from(floor),
            })
        })
        .collect()
}

pub(crate) fn ally_attack(combat: &mut CombatState, slot: usize) -> String {
    let Some(target) = first_target(&combat.enemies) else {
        combat.outcome = Some(CombatOutcome::Victory);
        return "No enemies remain.".to_owned();
    };
    let actor = combat.allies[slot].clone();
    let damage = attack_damage(&actor, &combat.enemies[target], 0);
    combat.enemies[target].hp -= damage;
    let summary = format!(
        "{} attacks {} for {} damage.",
        actor.name, combat.enemies[target].name, damage
    );
    combat.add_log(summary.clone());
    log_if_defeated(combat, CombatSide::Enemy, target);
    summary
}

pub(crate) fn ally_skill(combat: &mut CombatState, data: &GameData, slot: usize) -> String {
    let role = data
        .species(&combat.allies[slot].source_id)
        .map(|species| species.role)
        .unwrap_or(MonsterRole::Scout);
    match role {
        MonsterRole::Support => support_skill(combat, slot),
        MonsterRole::Tank => guard_back_row(combat, slot),
        MonsterRole::Scout => mark_target(combat, slot),
        MonsterRole::Striker => burst_strike(combat, slot),
    }
}

pub(crate) fn defend(combat: &mut CombatState, slot: usize) -> String {
    combat.allies[slot].is_defending = true;
    combat.allies[slot].is_guarding = false;
    let summary = format!("{} defends the formation.", combat.allies[slot].name);
    combat.add_log(summary.clone());
    summary
}

pub(crate) fn flee_chance(combat: &CombatState) -> i32 {
    let front_scouts = combat
        .allies
        .iter()
        .filter(|ally| ally.is_alive() && ally.slot < 3 && ally.role == Some(MonsterRole::Scout))
        .count() as i32;
    let marked_bonus = if combat
        .enemies
        .iter()
        .any(|enemy| enemy.is_alive() && enemy.is_marked)
    {
        15
    } else {
        0
    };
    (55 + front_scouts * 20 + marked_bonus).min(95)
}

pub(crate) fn flee_succeeds(combat: &CombatState) -> bool {
    let roll =
        ((combat.floor * 37 + combat.round * 17 + combat.turn_index as u32 * 11) % 100) as i32;
    roll < flee_chance(combat)
}

pub(crate) fn advance_to_player_or_outcome(combat: &mut CombatState) {
    for _ in 0..24 {
        if check_outcome(combat) {
            return;
        }
        let Some(turn) = combat.current_turn() else {
            rebuild_turn_order(combat);
            continue;
        };
        if !turn_is_alive(combat, turn) {
            advance_turn(combat);
            continue;
        }
        if turn.side == CombatSide::Ally {
            return;
        }
        enemy_action(combat, turn.slot);
        if check_outcome(combat) {
            return;
        }
        advance_turn(combat);
    }
}

pub(crate) fn advance_turn(combat: &mut CombatState) {
    combat.turn_index += 1;
    if combat.turn_index >= combat.turn_order.len() {
        combat.round += 1;
        for ally in &mut combat.allies {
            ally.is_defending = false;
            ally.is_guarding = false;
        }
        for enemy in &mut combat.enemies {
            enemy.is_defending = false;
        }
        rebuild_turn_order(combat);
    }
}

pub(crate) fn rebuild_turn_order(combat: &mut CombatState) {
    let mut turns = Vec::new();
    for (slot, ally) in combat.allies.iter().enumerate() {
        if ally.is_alive() {
            turns.push((
                ally.speed,
                CombatTurn {
                    side: CombatSide::Ally,
                    slot,
                },
            ));
        }
    }
    for (slot, enemy) in combat.enemies.iter().enumerate() {
        if enemy.is_alive() {
            turns.push((
                enemy.speed,
                CombatTurn {
                    side: CombatSide::Enemy,
                    slot,
                },
            ));
        }
    }
    turns.sort_by(|left, right| right.0.cmp(&left.0));
    combat.turn_order = turns.into_iter().map(|(_, turn)| turn).collect();
    combat.turn_index = 0;
}

pub(crate) fn sync_allies(state: &mut GameState, combat: &CombatState) {
    for ally in &combat.allies {
        if let Some(monster_id) = ally.monster_id {
            if let Some(monster) = state.monster_roster.monster_mut(monster_id) {
                monster.hp = ally.hp.max(1).min(monster.max_hp);
            }
        }
    }
}

pub(crate) fn award_xp(state: &mut GameState, xp_reward: u32) {
    for slot in state
        .monster_roster
        .party_slots
        .clone()
        .into_iter()
        .flatten()
    {
        if let Some(monster) = state.monster_roster.monster_mut(slot) {
            if monster.condition.commitment != DailyCommitment::Tower {
                continue;
            }
            monster.xp += xp_reward;
            while monster.xp >= monster.level * 20 {
                monster.xp -= monster.level * 20;
                monster.level += 1;
                monster.max_hp += 3;
                monster.hp = monster.max_hp;
                monster.attack += 1;
                monster.defense += 1;
                if monster.level % 2 == 0 {
                    monster.speed += 1;
                }
            }
        }
    }
}

pub(crate) fn combined_rewards(data: &GameData, enemies: &[Combatant]) -> Vec<ResourceStack> {
    let mut rewards = Vec::new();
    for combatant in enemies {
        let Some(enemy) = data.enemy(&combatant.source_id) else {
            continue;
        };
        for reward in &enemy.rewards {
            add_reward(&mut rewards, &reward.resource_id, reward.amount);
        }
    }
    rewards
}

pub(crate) fn encounter_xp(data: &GameData, enemies: &[Combatant]) -> u32 {
    enemies
        .iter()
        .filter_map(|enemy| data.enemy(&enemy.source_id))
        .map(|enemy| enemy.xp_reward)
        .sum()
}

pub(crate) fn record_floor_reached(state: &mut GameState, data: &GameData, floor: u32) {
    state.tower_progress.best_floor = state.tower_progress.best_floor.max(floor);
    if let Some(floor_data) = data.tower_floor(floor) {
        state.tower_progress.unlocked_floor = state
            .tower_progress
            .unlocked_floor
            .max(floor_data.unlocks_floor.max(floor))
            .min(max_floor(data));
    }
}

pub(crate) fn add_boss_egg(run: &mut TowerRunState, data: &GameData, floor: u32) {
    let Some(egg_type) = data.egg_type("boss_egg") else {
        return;
    };
    run.found_eggs.push(TowerFoundEgg {
        egg_type_id: egg_type.id.clone(),
        hatch_days: egg_type.hatch_days,
        origin_floor: floor,
        palette_seed: 0xB055_E66 ^ u64::from(floor),
    });
}

pub(crate) fn reward_text(data: &GameData, rewards: &[ResourceStack]) -> String {
    if rewards.is_empty() {
        return "no materials".to_owned();
    }
    rewards
        .iter()
        .map(|reward| {
            format!(
                "{} {}",
                reward.amount,
                data.resource_name(&reward.resource_id)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn victory_rewards(combat: &CombatState) -> Vec<ResourceStack> {
    let mut rewards = combat.rewards.clone();
    if combat.enemies.iter().any(|enemy| enemy.is_marked) {
        add_reward(&mut rewards, "coins", 2 + combat.floor as i32 / 2);
    }
    rewards
}

fn support_skill(combat: &mut CombatState, slot: usize) -> String {
    let wounded = combat
        .allies
        .iter()
        .enumerate()
        .filter(|(_, ally)| ally.is_alive())
        .filter(|(_, ally)| ally.hp < ally.max_hp)
        .max_by_key(|(_, ally)| ally.max_hp - ally.hp)
        .map(|(index, _)| index);

    let Some(target) = wounded else {
        return rally_ally(combat, slot);
    };

    let back_row_bonus = if combat.allies[slot].slot >= 3 { 3 } else { 0 };
    let heal = 8 + combat.allies[slot].level as i32 * 2 + back_row_bonus;
    combat.allies[target].hp = (combat.allies[target].hp + heal).min(combat.allies[target].max_hp);
    let summary = format!(
        "{} soothes {} for {} HP.",
        combat.allies[slot].name, combat.allies[target].name, heal
    );
    combat.add_log(summary.clone());
    summary
}

fn rally_ally(combat: &mut CombatState, slot: usize) -> String {
    let target = combat
        .allies
        .iter()
        .enumerate()
        .filter(|(_, ally)| ally.is_alive())
        .min_by_key(|(_, ally)| ally.morale)
        .map(|(index, _)| index)
        .unwrap_or(slot);
    combat.allies[target].morale += 8;
    combat.allies[target].speed += 1;
    let summary = format!(
        "{} steadies {} with a rallying call.",
        combat.allies[slot].name, combat.allies[target].name
    );
    combat.add_log(summary.clone());
    summary
}

fn guard_back_row(combat: &mut CombatState, slot: usize) -> String {
    combat.allies[slot].is_defending = true;
    combat.allies[slot].is_guarding = true;
    let summary = format!(
        "{} guards the back row and braces for impact.",
        combat.allies[slot].name
    );
    combat.add_log(summary.clone());
    summary
}

fn mark_target(combat: &mut CombatState, slot: usize) -> String {
    let Some(target) = first_target(&combat.enemies) else {
        combat.outcome = Some(CombatOutcome::Victory);
        return "No enemies remain.".to_owned();
    };
    combat.enemies[target].is_marked = true;
    combat.enemies[target].morale -= 5;
    let summary = format!(
        "{} marks {} for safer escape and better loot.",
        combat.allies[slot].name, combat.enemies[target].name
    );
    combat.add_log(summary.clone());
    summary
}

fn burst_strike(combat: &mut CombatState, slot: usize) -> String {
    let Some(target) = wounded_target(&combat.enemies).or_else(|| first_target(&combat.enemies))
    else {
        combat.outcome = Some(CombatOutcome::Victory);
        return "No enemies remain.".to_owned();
    };
    let actor = combat.allies[slot].clone();
    let wounded_bonus = if combat.enemies[target].hp < combat.enemies[target].max_hp {
        5
    } else {
        2
    };
    let execute_bonus = if combat.enemies[target].hp * 2 <= combat.enemies[target].max_hp {
        3
    } else {
        0
    };
    let damage = attack_damage(
        &actor,
        &combat.enemies[target],
        wounded_bonus + execute_bonus,
    );
    combat.enemies[target].hp -= damage;
    let summary = format!(
        "{} bursts into {} for {} damage.",
        actor.name, combat.enemies[target].name, damage
    );
    combat.add_log(summary.clone());
    log_if_defeated(combat, CombatSide::Enemy, target);
    summary
}

fn enemy_action(combat: &mut CombatState, slot: usize) {
    let Some((target, guarded_by)) = enemy_target(combat, slot) else {
        combat.outcome = Some(CombatOutcome::Defeat);
        return;
    };
    let actor = combat.enemies[slot].clone();
    let defending = combat.allies[target].is_defending || guarded_by.is_some();
    let damage = attack_damage(&actor, &combat.allies[target], 0);
    let damage = if defending {
        (damage / 2).max(1)
    } else {
        damage
    };
    combat.allies[target].hp -= damage;
    if let Some(guard_slot) = guarded_by {
        combat.add_log(format!(
            "{} intercepts {} for {} damage.",
            combat.allies[guard_slot].name, actor.name, damage
        ));
    } else {
        combat.add_log(format!(
            "{} hits {} for {} damage.",
            actor.name, combat.allies[target].name, damage
        ));
    }
    log_if_defeated(combat, CombatSide::Ally, target);
}

fn enemy_target(combat: &CombatState, enemy_slot: usize) -> Option<(usize, Option<usize>)> {
    let back_targeted = (combat.round + combat.floor + enemy_slot as u32) % 4 == 0;
    if back_targeted {
        if let Some(back_target) = row_target(&combat.allies, 3..6) {
            if let Some(guard) = guarding_front_tank(&combat.allies) {
                return Some((guard, Some(guard)));
            }
            return Some((back_target, None));
        }
    }
    first_target(&combat.allies).map(|target| (target, None))
}

fn first_target(combatants: &[Combatant]) -> Option<usize> {
    (0..3)
        .chain(3..6)
        .find(|slot| {
            combatants
                .iter()
                .any(|unit| unit.slot == *slot && unit.is_alive())
        })
        .and_then(|slot| {
            combatants
                .iter()
                .position(|unit| unit.slot == slot && unit.is_alive())
        })
}

fn row_target(combatants: &[Combatant], mut slots: std::ops::Range<usize>) -> Option<usize> {
    slots
        .find(|slot| {
            combatants
                .iter()
                .any(|unit| unit.slot == *slot && unit.is_alive())
        })
        .and_then(|slot| {
            combatants
                .iter()
                .position(|unit| unit.slot == slot && unit.is_alive())
        })
}

fn guarding_front_tank(combatants: &[Combatant]) -> Option<usize> {
    combatants.iter().position(|unit| {
        unit.is_alive() && unit.slot < 3 && unit.role == Some(MonsterRole::Tank) && unit.is_guarding
    })
}

fn wounded_target(combatants: &[Combatant]) -> Option<usize> {
    combatants
        .iter()
        .enumerate()
        .filter(|(_, unit)| unit.is_alive() && unit.hp < unit.max_hp)
        .min_by_key(|(_, unit)| unit.hp)
        .map(|(index, _)| index)
}

fn turn_is_alive(combat: &CombatState, turn: CombatTurn) -> bool {
    match turn.side {
        CombatSide::Ally => combat
            .allies
            .get(turn.slot)
            .is_some_and(Combatant::is_alive),
        CombatSide::Enemy => combat
            .enemies
            .get(turn.slot)
            .is_some_and(Combatant::is_alive),
    }
}

fn check_outcome(combat: &mut CombatState) -> bool {
    if combat.outcome.is_some() {
        return true;
    }
    if combat.enemies.iter().all(|enemy| !enemy.is_alive()) {
        combat.outcome = Some(CombatOutcome::Victory);
        combat.add_log("The enemies are defeated.".to_owned());
        return true;
    }
    if combat.allies.iter().all(|ally| !ally.is_alive()) {
        combat.outcome = Some(CombatOutcome::Defeat);
        combat.add_log("The party falls back defeated.".to_owned());
        return true;
    }
    false
}

fn attack_damage(actor: &Combatant, target: &Combatant, bonus: i32) -> i32 {
    let back_row_penalty = if actor.monster_id.is_some() && actor.slot >= 3 {
        1
    } else {
        0
    };
    let mark_bonus = if target.is_marked { 2 } else { 0 };
    (actor.attack + bonus + mark_bonus - back_row_penalty - target.defense / 2).max(1)
}

fn log_if_defeated(combat: &mut CombatState, side: CombatSide, slot: usize) {
    let name = match side {
        CombatSide::Ally => combat.allies.get(slot),
        CombatSide::Enemy => combat.enemies.get(slot),
    }
    .filter(|unit| unit.hp <= 0)
    .map(|unit| unit.name.clone());
    if let Some(name) = name {
        combat.add_log(format!("{name} is defeated."));
    }
}

fn add_reward(rewards: &mut Vec<ResourceStack>, resource_id: &str, amount: i32) {
    if let Some(existing) = rewards
        .iter_mut()
        .find(|reward| reward.resource_id == resource_id)
    {
        existing.amount += amount;
        return;
    }
    rewards.push(ResourceStack {
        resource_id: resource_id.to_owned(),
        amount,
    });
}

fn max_floor(data: &GameData) -> u32 {
    data.tower_floors
        .iter()
        .map(|floor| floor.floor)
        .max()
        .unwrap_or(1)
}

fn hash_id(id: &str) -> u64 {
    id.bytes().fold(0xC0B4_7000, |value, byte| {
        value.wrapping_mul(37) ^ u64::from(byte)
    })
}
