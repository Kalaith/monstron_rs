use serde::{Deserialize, Serialize};
use std::fmt;

use crate::state::ResourceStack;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerProgress {
    pub best_floor: u32,
    pub unlocked_floor: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerRunState {
    pub current_floor: u32,
    pub rooms_explored: u32,
    pub pressure: u32,
    pub pressure_limit: u32,
    #[serde(default)]
    pub goal: TowerRunGoal,
    #[serde(default)]
    pub map: TowerMapState,
    pub cargo: Vec<ResourceStack>,
    pub found_eggs: Vec<TowerFoundEgg>,
    pub event_log: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerTileKind {
    #[default]
    Wall,
    Floor,
    Corridor,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerMapObjectKind {
    Loot,
    Egg,
    Enemy,
    Boss,
    Stairs,
    Exit,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TowerRoom {
    pub start_x: u32,
    pub start_y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerMapObject {
    pub kind: TowerMapObjectKind,
    pub x: u32,
    pub y: u32,
    #[serde(default)]
    pub resource_id: String,
    #[serde(default)]
    pub amount: i32,
    #[serde(default)]
    pub egg_type_id: String,
    #[serde(default)]
    pub hatch_days: u32,
    #[serde(default)]
    pub palette_seed: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerMapState {
    pub floor: u32,
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub player_x: u32,
    pub player_y: u32,
    pub start_x: u32,
    pub start_y: u32,
    pub tiles: Vec<TowerTileKind>,
    #[serde(default)]
    pub visibility: Vec<TowerTileVisibility>,
    pub rooms: Vec<TowerRoom>,
    pub objects: Vec<TowerMapObject>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerTileVisibility {
    #[default]
    Hidden,
    Explored,
    Visible,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum TowerRunGoal {
    #[default]
    Balanced,
    EggHunt,
    Salvage,
    Scout,
    PushDeeper,
    SafeRun,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerFoundEgg {
    pub egg_type_id: String,
    pub hatch_days: u32,
    pub origin_floor: u32,
    pub palette_seed: u64,
}

impl TowerRunState {
    pub fn new(current_floor: u32, pressure_limit: u32, goal: TowerRunGoal) -> Self {
        Self {
            current_floor,
            rooms_explored: 0,
            pressure: 0,
            pressure_limit,
            goal,
            map: TowerMapState::empty(),
            cargo: Vec::new(),
            found_eggs: Vec::new(),
            event_log: vec![format!("Entered floor {current_floor} on a {goal} run.")],
        }
    }

    pub fn with_map(mut self, map: TowerMapState) -> Self {
        self.map = map;
        self
    }

    pub fn add_cargo(&mut self, resource_id: &str, amount: i32) {
        if let Some(stack) = self
            .cargo
            .iter_mut()
            .find(|stack| stack.resource_id == resource_id)
        {
            stack.amount += amount;
            return;
        }

        self.cargo.push(ResourceStack {
            resource_id: resource_id.to_owned(),
            amount,
        });
    }

    pub fn cargo_amount(&self) -> i32 {
        self.cargo.iter().map(|stack| stack.amount.max(0)).sum()
    }

    pub fn add_event(&mut self, message: String) {
        self.event_log.push(message);
        if self.event_log.len() > 7 {
            let overflow = self.event_log.len() - 7;
            self.event_log.drain(0..overflow);
        }
    }
}

impl TowerTileKind {
    pub fn is_passable(self) -> bool {
        matches!(self, Self::Floor | Self::Corridor)
    }
}

impl TowerRoom {
    pub fn center(self) -> (u32, u32) {
        (
            self.start_x + self.width / 2,
            self.start_y + self.height / 2,
        )
    }

    pub fn random_inner(self, rng: &mut TowerMapRng) -> (u32, u32) {
        let min_x = self.start_x + 1;
        let max_x = (self.start_x + self.width - 1).max(min_x + 1);
        let min_y = self.start_y + 1;
        let max_y = (self.start_y + self.height - 1).max(min_y + 1);
        (rng.range(min_x, max_x), rng.range(min_y, max_y))
    }

    pub fn intersects_padded(self, other: Self) -> bool {
        let left = self.start_x.saturating_sub(1);
        let right = self.start_x + self.width + 1;
        let top = self.start_y.saturating_sub(1);
        let bottom = self.start_y + self.height + 1;

        left <= other.start_x + other.width
            && right >= other.start_x
            && top <= other.start_y + other.height
            && bottom >= other.start_y
    }
}

impl TowerMapState {
    pub fn empty() -> Self {
        Self {
            floor: 0,
            width: 0,
            height: 0,
            seed: 0,
            player_x: 0,
            player_y: 0,
            start_x: 0,
            start_y: 0,
            tiles: Vec::new(),
            visibility: Vec::new(),
            rooms: Vec::new(),
            objects: Vec::new(),
        }
    }

    pub fn new(width: u32, height: u32, floor: u32, seed: u64) -> Self {
        Self {
            floor,
            width,
            height,
            seed,
            player_x: 0,
            player_y: 0,
            start_x: 0,
            start_y: 0,
            tiles: vec![TowerTileKind::Wall; (width * height) as usize],
            visibility: vec![TowerTileVisibility::Hidden; (width * height) as usize],
            rooms: Vec::new(),
            objects: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0 || self.tiles.is_empty()
    }

    pub fn tile_at(&self, x: u32, y: u32) -> TowerTileKind {
        self.index(x, y)
            .and_then(|index| self.tiles.get(index).copied())
            .unwrap_or(TowerTileKind::Wall)
    }

    pub fn set_tile(&mut self, x: u32, y: u32, tile: TowerTileKind) {
        if let Some(index) = self.index(x, y) {
            if let Some(slot) = self.tiles.get_mut(index) {
                *slot = tile;
            }
        }
    }

    pub fn is_passable(&self, x: u32, y: u32) -> bool {
        self.tile_at(x, y).is_passable()
    }

    pub fn ensure_visibility(&mut self) -> bool {
        let expected_len = (self.width * self.height) as usize;
        if expected_len == 0 || self.visibility.len() == expected_len {
            return false;
        }

        self.visibility = vec![TowerTileVisibility::Hidden; expected_len];
        true
    }

    pub fn visibility_at(&self, x: u32, y: u32) -> TowerTileVisibility {
        self.index(x, y)
            .and_then(|index| self.visibility.get(index).copied())
            .unwrap_or(TowerTileVisibility::Hidden)
    }

    pub fn set_visibility(&mut self, x: u32, y: u32, visibility: TowerTileVisibility) {
        if let Some(index) = self.index(x, y) {
            if let Some(slot) = self.visibility.get_mut(index) {
                *slot = visibility;
            }
        }
    }

    pub fn is_visible(&self, x: u32, y: u32) -> bool {
        self.visibility_at(x, y) == TowerTileVisibility::Visible
    }

    pub fn is_discovered(&self, x: u32, y: u32) -> bool {
        matches!(
            self.visibility_at(x, y),
            TowerTileVisibility::Explored | TowerTileVisibility::Visible
        )
    }

    pub fn object_index_at(&self, x: u32, y: u32) -> Option<usize> {
        self.objects
            .iter()
            .position(|object| object.x == x && object.y == y)
    }

    pub fn object_at(&self, x: u32, y: u32) -> Option<&TowerMapObject> {
        self.objects
            .iter()
            .find(|object| object.x == x && object.y == y)
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }
}

impl Default for TowerMapState {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Clone, Debug)]
pub struct TowerMapRng {
    state: u64,
}

impl TowerMapRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.state >> 32) as u32
    }

    pub fn range(&mut self, min: u32, max_exclusive: u32) -> u32 {
        if max_exclusive <= min {
            return min;
        }
        min + self.next_u32() % (max_exclusive - min)
    }

    pub fn chance(&mut self, numerator: u32, denominator: u32) -> bool {
        denominator == 0 || self.range(0, denominator) < numerator
    }
}

impl TowerRunGoal {
    pub const CHOICES: [Self; 5] = [
        Self::EggHunt,
        Self::Salvage,
        Self::Scout,
        Self::PushDeeper,
        Self::SafeRun,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Balanced => "Balanced",
            Self::EggHunt => "Egg Hunt",
            Self::Salvage => "Salvage",
            Self::Scout => "Scout",
            Self::PushDeeper => "Push",
            Self::SafeRun => "Safe Run",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Self::Balanced => "Normal eggs, loot, danger, and floor progress.",
            Self::EggHunt => "More egg rooms and nests; material caches are smaller.",
            Self::Salvage => "More wood, stone, ore, and coins; egg finds are rarer.",
            Self::Scout => "Fewer enemies and more open routes; rewards are modest.",
            Self::PushDeeper => "More stairs and deeper routes; enemies are denser.",
            Self::SafeRun => "Fewer enemies and traps; fewer rewards.",
        }
    }
}

impl fmt::Display for TowerRunGoal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}
