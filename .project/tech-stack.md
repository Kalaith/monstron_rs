# Hatchspire - Tech Stack

## Runtime

- Language: Rust 2021.
- Primary engine: Macroquad.
- Shared helper crate: `macroquad-toolkit`.
- Targets: Windows desktop and WebGL.
- Persistence: `quad-storage` for browser-compatible local saves, with native fallback through the same save abstraction.

## Core Dependencies

Initial `Cargo.toml` should follow the style used by nearby RustGames projects:

```toml
[package]
name = "monstron"
version = "0.1.0"
edition = "2021"

[dependencies]
macroquad = { version = "0.4", features = ["audio"] }
macroquad-toolkit = { path = "../macroquad-toolkit" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
quad-storage = "0.1"
```

If WebGL compatibility conflicts with `rand`, prefer the random approach already used successfully in sibling projects.

## Local References

| Project | Use As Reference For |
| --- | --- |
| `alchemy_tower` | Embedded JSON data loading, state transitions, content organization |
| `quiteville` | Town resources, zones, activity log, camera/UI patterns |
| `dungeon_manager_2d` | Grid maps, pathing ideas, combat stat modeling |
| `cultivation` | Larger simulation structure and long-term scheduler ideas |

## Architecture

Planned source layout:

```text
src/
  main.rs
  game.rs
  data/
    mod.rs
    schema.rs
    loader.rs
    game_data.rs
  state/
    mod.rs
    app_state.rs
    town.rs
    monsters.rs
    eggs.rs
    tower.rs
    combat.rs
  engine/
    mod.rs
    day_engine.rs
    town_engine.rs
    monster_engine.rs
    egg_engine.rs
    tower_generator.rs
    combat_engine.rs
  screens/
    mod.rs
    main_menu.rs
    town.rs
    hatchery.rs
    stable.rs
    dungeon_prep.rs
    tower.rs
    combat.rs
    end_of_day.rs
  ui/
    mod.rs
    theme.rs
    widgets.rs
    panels.rs
  save/
    mod.rs
    save_data.rs
  assets/
    mod.rs
    art.rs
```

Data files:

```text
assets/data/
  buildings.json
  resources.json
  monster_species.json
  egg_types.json
  tower_floors.json
  enemies.json
  combat_skills.json
  npcs.json
  story_flags.json
```

## State Model

Use one top-level `GameState` for persistent simulation state:

- `day`
- `resources`
- `town`
- `monster_roster`
- `egg_inventory`
- `tower_progress`
- `npc_relationships`
- `story_flags`
- `activity_log`

Use short-lived screen state for UI selection, hover state, modal state, combat animation timers, and pending actions.

## Screen Flow

```text
MainMenu
  -> Town
    -> Hatchery
    -> Stable
    -> Workshop
    -> Shop
    -> DungeonPrep
      -> TowerExplore
        -> Combat
        -> LootSummary
    -> EndOfDay
```

## Data Loading Strategy

Prefer `alchemy_tower`'s embedded JSON approach for MVP so WebGL builds do not depend on runtime file access:

```rust
const BUILDINGS_JSON: &str = include_str!("../assets/data/buildings.json");
```

The loader should validate:

- Unique IDs.
- References to existing resources, species, skills, buildings, and floors.
- No empty required display names.
- No negative costs, health, or rewards.

## Combat Model

The combat engine should support six allied slots from the start:

```text
Back:  [3] [4] [5]
Front: [0] [1] [2]
```

MVP can allow three active monsters, but the data structure should already support three front and three back slots. Front slots are targetable before matching back slots unless skills override targeting.

## Asset Strategy

MVP art should be simple, deterministic, and shippable:

- Macroquad shapes and generated pixel sprites for early development.
- Data-driven palettes per monster species and element.
- Later optional AI-assisted monster image generation should be an offline content pipeline, not a runtime dependency.

The future AI-art pipeline should take inherited monster attributes and produce curated sprite candidates. The game should store final approved sprites as assets.

## Build Commands

After Phase 0 scaffolding:

```powershell
cargo check -p monstron
cargo build -p monstron
cargo build -p monstron --target wasm32-unknown-unknown --release
.\publish.ps1
```

No build is expected to pass before `Cargo.toml` and `src/` are created.
