use serde::{Deserialize, Serialize};

use crate::data::{
    Element, GameData, MonsterRole, MonsterSpeciesDefinition, PassiveSkill, Temperament, TownSkill,
};
use crate::state::MonsterArtProfile;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MonsterRoster {
    pub next_id: u64,
    pub monsters: Vec<MonsterInstance>,
    pub party_slots: Vec<Option<u64>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MonsterInstance {
    pub id: u64,
    pub name: String,
    pub species_id: String,
    pub element: Element,
    pub temperament: Temperament,
    pub role: MonsterRole,
    pub passive: PassiveSkill,
    pub town_skill: TownSkill,
    pub level: u32,
    pub xp: u32,
    pub bond: u32,
    pub max_hp: i32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub visual_seed: u64,
    #[serde(default)]
    pub condition: MonsterCondition,
    #[serde(default)]
    pub art_profile: MonsterArtProfile,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum DailyCommitment {
    #[default]
    Free,
    Tower,
    Breeding,
    Rest,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MonsterCondition {
    pub fatigue: u32,
    pub injury_days: u32,
    #[serde(default)]
    pub commitment: DailyCommitment,
}

impl MonsterInstance {
    pub fn from_species(
        id: u64,
        name: String,
        species: &MonsterSpeciesDefinition,
        visual_seed: u64,
    ) -> Self {
        Self {
            id,
            name,
            species_id: species.id.clone(),
            element: species.element,
            temperament: species.temperament,
            role: species.role,
            passive: species.passive,
            town_skill: species.town_skill,
            level: 1,
            xp: 0,
            bond: 1,
            max_hp: species.base_hp,
            hp: species.base_hp,
            attack: species.base_attack,
            defense: species.base_defense,
            speed: species.base_speed,
            visual_seed,
            condition: MonsterCondition::default(),
            art_profile: MonsterArtProfile::from_traits(
                species,
                species.element,
                species.temperament,
                species.passive,
                species.town_skill,
                visual_seed,
            ),
        }
    }

    pub fn is_injured(&self) -> bool {
        self.condition.injury_days > 0
    }

    pub fn is_battle_ready(&self) -> bool {
        self.hp > 0 && !self.is_injured()
    }

    pub fn fatigue_penalty(&self) -> i32 {
        self.condition.fatigue.div_ceil(2).min(3) as i32
    }

    pub fn refresh_art_profile(&mut self, species: &MonsterSpeciesDefinition) {
        self.art_profile = MonsterArtProfile::from_traits(
            species,
            self.element,
            self.temperament,
            self.passive,
            self.town_skill,
            self.visual_seed,
        );
    }
}

impl MonsterRoster {
    pub fn add_monster(
        &mut self,
        name: String,
        species: &MonsterSpeciesDefinition,
        seed: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.monsters
            .push(MonsterInstance::from_species(id, name, species, seed));
        id
    }

    pub fn is_in_party(&self, monster_id: u64) -> bool {
        self.party_slots
            .iter()
            .any(|slot| slot.is_some_and(|id| id == monster_id))
    }

    pub fn first_empty_party_slot(&self) -> Option<usize> {
        self.party_slots.iter().position(Option::is_none)
    }

    pub fn assign_to_party(&mut self, monster_id: u64) -> Result<usize, String> {
        if !self.monsters.iter().any(|monster| monster.id == monster_id) {
            return Err("Monster is not in the roster.".to_owned());
        }

        if let Some(index) = self
            .party_slots
            .iter()
            .position(|slot| slot.is_some_and(|id| id == monster_id))
        {
            return Ok(index);
        }

        let Some(index) = self.first_empty_party_slot() else {
            return Err("The six-slot party is full.".to_owned());
        };

        self.party_slots[index] = Some(monster_id);
        Ok(index)
    }

    pub fn remove_from_party(&mut self, slot_index: usize) -> Option<u64> {
        self.party_slots.get_mut(slot_index).and_then(Option::take)
    }

    pub fn monster(&self, monster_id: u64) -> Option<&MonsterInstance> {
        self.monsters
            .iter()
            .find(|monster| monster.id == monster_id)
    }

    pub fn monster_mut(&mut self, monster_id: u64) -> Option<&mut MonsterInstance> {
        self.monsters
            .iter_mut()
            .find(|monster| monster.id == monster_id)
    }

    pub fn ensure_art_profiles(&mut self, data: &GameData) {
        for monster in &mut self.monsters {
            if !monster.art_profile.is_empty() {
                continue;
            }
            if let Some(species) = data.species(&monster.species_id) {
                monster.refresh_art_profile(species);
            }
        }
    }
}
