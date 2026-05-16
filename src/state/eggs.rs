use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EggInventory {
    pub next_id: u64,
    pub eggs: Vec<EggInstance>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EggInstance {
    pub id: u64,
    pub egg_type_id: String,
    pub days_remaining: u32,
    pub origin_floor: u32,
    pub palette_seed: u64,
}

impl EggInventory {
    pub fn add_egg(
        &mut self,
        egg_type_id: String,
        days_remaining: u32,
        origin_floor: u32,
        palette_seed: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.eggs.push(EggInstance {
            id,
            egg_type_id,
            days_remaining,
            origin_floor,
            palette_seed,
        });
        id
    }

    pub fn egg_mut(&mut self, egg_id: u64) -> Option<&mut EggInstance> {
        self.eggs.iter_mut().find(|egg| egg.id == egg_id)
    }

    pub fn remove_egg(&mut self, egg_id: u64) -> Option<EggInstance> {
        let index = self.eggs.iter().position(|egg| egg.id == egg_id)?;
        Some(self.eggs.remove(index))
    }
}
