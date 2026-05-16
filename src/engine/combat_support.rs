use crate::data::{EnemyDefinition, GameData, MonsterRole};
use crate::state::{
    CombatOutcome, CombatSide, CombatState, CombatTurn, Combatant, GameState, ResourceStack,
    TowerFoundEgg, TowerRunState,
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
            let penalty = monster.fatigue_penalty();
            let max_hp = (monster.max_hp - penalty * 2).max(1);
            Some(Combatant {
                name: monster.name.clone(),
                source_id: monster.species_id.clone(),
                monster_id: Some(monster.id),
                slot,
                level: monster.level,
                max_hp,
                hp: monster.hp.min(max_hp),
                attack: (monster.attack - penalty).max(1),
                defense: (monster.defense - penalty).max(0),
                speed: (monster.speed - penalty).max(1),
                morale: 50 + monster.bond as i32 - monster.condition.fatigue as i32 * 4,
                is_defending: false,
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
                slot,
                level: floor,
                max_hp: enemy.max_hp + floor as i32,
                hp: enemy.max_hp + floor as i32,
                attack: enemy.attack + floor as i32 / 3,
                defense: enemy.defense,
                speed: enemy.speed,
                morale: if is_boss { 90 } else { 35 + floor as i32 * 3 },
                is_defending: false,
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
    let damage = damage_amount(actor.attack, combat.enemies[target].defense, false);
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
        MonsterRole::Support => heal_lowest_ally(combat, slot),
        MonsterRole::Tank => defend(combat, slot),
        MonsterRole::Scout => skill_strike(combat, slot, 4, "quick strike"),
        MonsterRole::Striker => skill_strike(combat, slot, 7, "heavy strike"),
    }
}

pub(crate) fn defend(combat: &mut CombatState, slot: usize) -> String {
    combat.allies[slot].is_defending = true;
    let summary = format!("{} defends the formation.", combat.allies[slot].name);
    combat.add_log(summary.clone());
    summary
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

fn heal_lowest_ally(combat: &mut CombatState, slot: usize) -> String {
    let Some(target) = combat
        .allies
        .iter()
        .enumerate()
        .filter(|(_, ally)| ally.is_alive())
        .min_by_key(|(_, ally)| ally.hp)
        .map(|(index, _)| index)
    else {
        return "No allies can be healed.".to_owned();
    };
    let heal = 8 + combat.allies[slot].level as i32 * 2;
    combat.allies[target].hp = (combat.allies[target].hp + heal).min(combat.allies[target].max_hp);
    let summary = format!(
        "{} soothes {} for {} HP.",
        combat.allies[slot].name, combat.allies[target].name, heal
    );
    combat.add_log(summary.clone());
    summary
}

fn skill_strike(combat: &mut CombatState, slot: usize, bonus: i32, label: &str) -> String {
    let Some(target) = first_target(&combat.enemies) else {
        combat.outcome = Some(CombatOutcome::Victory);
        return "No enemies remain.".to_owned();
    };
    let actor = combat.allies[slot].clone();
    let damage = damage_amount(actor.attack + bonus, combat.enemies[target].defense, false);
    combat.enemies[target].hp -= damage;
    let summary = format!(
        "{} uses {} on {} for {} damage.",
        actor.name, label, combat.enemies[target].name, damage
    );
    combat.add_log(summary.clone());
    log_if_defeated(combat, CombatSide::Enemy, target);
    summary
}

fn enemy_action(combat: &mut CombatState, slot: usize) {
    let Some(target) = first_target(&combat.allies) else {
        combat.outcome = Some(CombatOutcome::Defeat);
        return;
    };
    let actor = combat.enemies[slot].clone();
    let defending = combat.allies[target].is_defending;
    let damage = damage_amount(actor.attack, combat.allies[target].defense, defending);
    combat.allies[target].hp -= damage;
    combat.add_log(format!(
        "{} hits {} for {} damage.",
        actor.name, combat.allies[target].name, damage
    ));
    log_if_defeated(combat, CombatSide::Ally, target);
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

fn damage_amount(attack: i32, defense: i32, defending: bool) -> i32 {
    let damage = (attack - defense / 2).max(1);
    if defending {
        (damage / 2).max(1)
    } else {
        damage
    }
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
