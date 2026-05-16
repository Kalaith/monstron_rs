use serde::{Deserialize, Serialize};

use crate::data::{Element, MonsterSpeciesDefinition, PassiveSkill, Temperament, TownSkill};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MonsterArtProfile {
    pub species_hint: String,
    pub silhouette: String,
    pub palette: String,
    pub markings: String,
    pub accessory: String,
    pub mood: String,
    pub lineage_code: String,
}

impl MonsterArtProfile {
    pub fn from_traits(
        species: &MonsterSpeciesDefinition,
        element: Element,
        temperament: Temperament,
        passive: PassiveSkill,
        town_skill: TownSkill,
        seed: u64,
    ) -> Self {
        Self {
            species_hint: species.name.clone(),
            silhouette: silhouette_for_species(&species.id).to_owned(),
            palette: palette_for_element(element, seed).to_owned(),
            markings: markings_for_passive(passive).to_owned(),
            accessory: accessory_for_town_skill(town_skill).to_owned(),
            mood: mood_for_temperament(temperament).to_owned(),
            lineage_code: format!("{:08x}", seed as u32),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.species_hint.is_empty()
            && self.silhouette.is_empty()
            && self.palette.is_empty()
            && self.markings.is_empty()
            && self.accessory.is_empty()
            && self.mood.is_empty()
    }
}

fn silhouette_for_species(species_id: &str) -> &'static str {
    match species_id {
        "slime" => "round translucent slime body, soft droplet ears, tiny feet",
        "rootling" => "small root creature, leafy crown, sturdy stump legs",
        "glowmoth" => "mothlike sprite, wide soft wings, glowing antennae",
        "pebblepup" => "compact stone puppy, chunky paws, gem nose",
        "emberkit" => "small ember fox kit, flame tail, alert ears",
        "rillfin" => "gentle finned amphibian, flowing whiskers, water fins",
        _ => "small friendly tower-born monster, readable compact silhouette",
    }
}

fn palette_for_element(element: Element, seed: u64) -> &'static str {
    match (element, seed % 3) {
        (Element::Water, 0) => "aqua, teal, pearl, and deep blue palette",
        (Element::Water, 1) => "seafoam, cyan, indigo, and pale mint palette",
        (Element::Water, _) => "rain blue, soft violet, and glassy turquoise palette",
        (Element::Fire, 0) => "ember orange, warm gold, charcoal, and coral palette",
        (Element::Fire, 1) => "rose red, brass gold, smoke gray, and cream palette",
        (Element::Fire, _) => "lantern yellow, burnt sienna, and crimson palette",
        (Element::Earth, 0) => "moss green, bark brown, clay, and pale stone palette",
        (Element::Earth, 1) => "fern green, ochre, slate, and ivory palette",
        (Element::Earth, _) => "sage, mineral gray, root brown, and amber palette",
    }
}

fn markings_for_passive(passive: PassiveSkill) -> &'static str {
    match passive {
        PassiveSkill::FindsSmallLoot => "tiny coin-like freckles and bright curious eyes",
        PassiveSkill::ResistsPoison => "protective leaf-vein markings and hardy shell patches",
        PassiveSkill::DetectsEggs => "soft glowing speckles and nest-shaped motifs",
        PassiveSkill::FindsStone => "mineral chips, pebble spots, and sturdy facets",
        PassiveSkill::BurnsBrambles => "warm ember streaks and singed vine patterns",
        PassiveSkill::SoothesInjuries => "gentle healer markings and soft crescent highlights",
    }
}

fn accessory_for_town_skill(town_skill: TownSkill) -> &'static str {
    match town_skill {
        TownSkill::HatcheryHelper => "small hatchery charm tied with twine",
        TownSkill::Farming => "sprout charm and garden soil details",
        TownSkill::Lighting => "tiny lantern glow accents",
        TownSkill::Guarding => "simple camp guard neckerchief",
        TownSkill::WorkshopHeat => "little brass workshop tag and warm sparks",
        TownSkill::Hatching => "egg-warming sash and soft nest feathers",
    }
}

fn mood_for_temperament(temperament: Temperament) -> &'static str {
    match temperament {
        Temperament::Loyal => "loyal, alert, and companionable expression",
        Temperament::Patient => "calm, grounded, patient expression",
        Temperament::Curious => "curious, bright, upward-looking expression",
        Temperament::Brave => "brave stance, squared posture, confident eyes",
        Temperament::Restless => "restless energy, playful lean, lively eyes",
        Temperament::Gentle => "gentle healer expression, relaxed posture",
    }
}
