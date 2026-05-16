# Hatchspire Rust Conversion Plan

## Purpose

This plan defines how the old Unity Monstron idea becomes a new Rust/Macroquad game in `H:\WebHatchery\RustGames\monstron`.

The Unity project is concept reference only. The Rust version should keep the strongest ideas:

- Monsters collect eggs and resources from a dungeon.
- Eggs become new monsters.
- Monsters feed town growth.
- Town growth unlocks deeper dungeon progression.
- Monster traits and generated visuals make each creature feel personal.

The Rust version should not copy Unity scene structure, controller names, UI layout, or old implementation constraints unless they still make sense.

## Recommended Direction

Use the working title **Hatchspire** for planning and in-game presentation while keeping the Rust crate/package name `monstron`.

The first product target is a compact playable MVP:

- 6 town buildings.
- 3 NPCs.
- 6 monster species.
- 3 elements.
- 12 egg types.
- 10 tower floors.
- 8 enemy types.
- 1 boss.
- Turn-based combat with a six-slot party model.
- Basic hatching and early breeding.

## Reference Projects In RustGames

### `alchemy_tower`

Use for:

- Embedded JSON content with `include_str!`.
- Clean `Game` ownership of state, data, assets, and transitions.
- Content loader validation and lookup maps.

Why it matters:

Hatchspire will have many definitions: buildings, monsters, eggs, floors, enemies, skills, NPCs, and resources. `alchemy_tower` already shows a good pattern for loading that kind of data without relying on runtime file access.

### `quiteville`

Use for:

- Town-builder resource state.
- Activity logs.
- Town view organization.
- Selection and inspection UI.

Why it matters:

The town should feel like a living hub that reacts to resources, buildings, NPCs, and daily progress.

### `dungeon_manager_2d`

Use for:

- Grid and map generation ideas.
- Combat stat extraction.
- Creature targeting and pathing references.

Why it matters:

The battle system is turn-based instead of real-time, but the project has useful examples for separating creature data from engine behavior.

### `cultivation`

Use later for:

- Larger simulation scheduling.
- Long-term progression systems.
- Multi-system state persistence.

Why it matters:

Hatchspire may eventually need daily jobs, passive monster work, NPC schedules, festivals, and multi-day breeding/hatching timers.

## System Mapping From Unity Inspiration

| Unity Inspiration | Rust Direction |
| --- | --- |
| `GameController` style global object | `Game` plus explicit `GameState` |
| Town scene and controllers | `screens::town` and `engine::town_engine` |
| Dungeon scene | `screens::tower` and `engine::tower_generator` |
| Monster corral/collection | `state::monsters::MonsterRoster` |
| Enemy controller logic | `engine::combat_engine` and enemy data |
| Pixel monster generation | `assets::art` placeholder generator, later offline art pipeline |
| Scene transitions | `AppScreen` enum and transition actions |
| Unity serialized data | JSON data under `assets/data/` |

## Architecture Plan

### Data Layer

Purpose: load and validate static game definitions.

Primary files:

- `buildings.json`
- `resources.json`
- `monster_species.json`
- `egg_types.json`
- `tower_floors.json`
- `enemies.json`
- `combat_skills.json`
- `npcs.json`

Rust modules:

- `data::schema`
- `data::loader`
- `data::game_data`

Rules:

- Static data uses stable string IDs.
- Runtime save data stores IDs, not duplicated static definitions.
- Loader validates references before the game starts.

### State Layer

Purpose: store the current save state.

Core state:

- `day`
- `resources`
- `town`
- `monster_roster`
- `egg_inventory`
- `tower_progress`
- `npc_relationships`
- `story_flags`
- `activity_log`

State should be serializable. Runtime-only UI data should not be saved.

### Engine Layer

Purpose: mutate state through explicit rules.

Engines:

- `day_engine`: sleep, daily ticks, recovery, hatch timers.
- `town_engine`: build, upgrade, spend resources, unlock facilities.
- `monster_engine`: XP, level, bond, recovery, town jobs.
- `egg_engine`: egg care, hatching, inheritance.
- `tower_generator`: floor events, loot, encounter generation.
- `combat_engine`: turn order, actions, targeting, rewards.

### Screen/UI Layer

Purpose: draw state and collect player intent.

Screens:

- Main menu.
- Town.
- Hatchery.
- Stable.
- Breeding Grove.
- Workshop.
- Shop.
- Dungeon prep.
- Tower exploration.
- Combat.
- End-of-day summary.

UI functions should return actions such as `TownAction::Build("hatchery")` rather than directly mutating state.

## Core Gameplay Loop For MVP

```text
New Day
  -> Review town, monsters, eggs, resources
  -> Choose building or hatchery actions
  -> Prepare monster party
  -> Enter tower
  -> Explore rooms and fight battles
  -> Collect eggs and materials
  -> Return to town
  -> Spend rewards on hatching and building
  -> Sleep
```

Every loop should produce at least one of:

- A stronger town.
- A new or improved monster.
- Deeper tower access.
- A clearer future goal.

## Combat Plan

Support six allied slots immediately:

```text
Back row:  [3] [4] [5]
Front row: [0] [1] [2]
```

MVP behavior:

- Front monsters are targeted before back monsters.
- Back monsters can use support or ranged skills.
- Speed determines turn order.
- Player commands monster actions.
- Enemies use simple weighted AI.
- Defeat returns the party to town with penalties, not permanent loss.

Initial actions:

- Attack.
- Species skill.
- Defend.
- Item.
- Flee.

The starter slime begins weak but useful:

- Low damage.
- Good escape/scout utility.
- Can find extra small loot.
- Helps stabilize egg hatching later.

## Monster And Egg Plan

MVP monster data:

- Species.
- Element.
- Temperament.
- Role.
- Passive skill.
- Town skill.
- Bond.
- Level and XP.
- Generated visual seed.

MVP egg data:

- Egg type.
- Rarity.
- Possible species.
- Element bias.
- Temperament bias.
- Hatch timer.
- Discovery floor.
- Visual palette seed.

Breeding should wait until hatching and combat are stable. When added, it should create an egg rather than directly creating a monster.

Inheritance model:

- Species family from either parent or compatible hybrid table.
- Element from either parent with small mutation chance.
- Temperament weighted from parents.
- Passive from either parent, tower origin, or mutation.
- Visual seed generated from both parents plus egg origin.

## AI-Assisted Monster Art Plan

Do not make runtime AI art part of the MVP.

Recommended future pipeline:

1. Runtime breeding creates a full child attribute profile.
2. A developer tool converts the profile into an art prompt.
3. AI generates candidate sprites or portraits offline.
4. The developer curates and imports approved assets.
5. The game stores only asset IDs and deterministic fallback visuals.

This keeps the game shippable offline and avoids depending on API availability during play.

## First Implementation Pass

The first coding pass should build the smallest running Rust app:

1. [x] Add the crate to the RustGames workspace.
2. [x] Open to a title screen.
3. [x] Start a new save.
4. [x] Show a town screen with day, resources, buildings, monster roster, and activity log.
5. [x] Include a starter slime.
6. [x] Allow sleep/save/load.

No dungeon or combat should be implemented until this shell is stable.

## Build And Quality Gates

After crate scaffolding:

```powershell
cargo check -p monstron
cargo fmt -p monstron
cargo build -p monstron
cargo build -p monstron --target wasm32-unknown-unknown --release
```

Status on 2026-05-16: the scaffold and vertical slice shell are in place, and the listed format/check/build commands pass. `.\publish.ps1 -WebGLOnly -SkipBuild -DryRun` also packages against the workspace target directory. Dungeon prep and end-of-day remain lightweight transition screens; tower exploration and combat are implemented by the later MVP phases below.

Phase 2 status on 2026-05-16: the town MVP can build and upgrade the five MVP buildings, validates resource spending, shows town rank, exposes three NPC service stubs with friendship greetings, and enables simple shop buy/sell trades once the shop is built. A temporary `Scavenge` action supplies resources until tower loot exists.

Phase 3 status on 2026-05-16: monster traits are typed as elements, temperaments, roles, passives, and town skills; all 12 egg definitions include rarity, species, element, and temperament bias; the hatchery can recover fallback eggs, warm them with herbs, and hatch deterministic monsters into the roster; the stable can inspect monsters and manage the six-slot party.

Phase 4 status on 2026-05-16: tower floor data now defines 10 floors with loot, egg pools, pressure limits, exit progression, and a boss-floor marker; tower runs track current floor, explored rooms, pressure, cargo, found eggs, event logs, best floor, and floor unlocks; the tower screen lets the party explore loot, egg, enemy, exit, and boss events, then return to town with a loot summary that deposits materials and eggs. Enemy and boss events now hand off to Phase 5 combat.

Phase 5 status on 2026-05-16: tower enemy and boss events now start turn-based combat using three front and three back allied slots, enemy formation slots, HP/attack/defense/speed/morale stats, speed-based turn order, action choices for attack, role skill, defend, herbs, and flee, front-row shielding target rules, eight enemy definitions plus the floor-10 Verdant Crown boss, combat rewards, monster XP and level growth, victory returns to the tower run, flee returns current cargo to town, and defeat rescues the party to town while dropping run cargo.

Phase 6 status on 2026-05-16: Breeding Grove is now a buildable town facility with a dedicated screen, pair compatibility checks based on shared element, shared role, or combined bond, herb-based pairing costs, parent bond gains, a starter slime bond story hook, and bred eggs that persist inherited species family, element, temperament, passive, palette seed, parent IDs, origin floor, and floor-scaled mutation state. Hatching now applies inherited lineage data and reports inherited or mutated children.

Phase 7 status on 2026-05-16: Workshop and Shop are now dedicated focused screens instead of only town-panel stubs. Workshop assigns monsters to persistent town jobs, including forage, quarry, workshop support, and egg care; daily sleep now resolves those jobs into resources, egg warmth, and bond growth using each monster's role, passive, and town skill. Shop has its own trade interface for resource conversion. Routing was split so facility action handlers stay outside `game.rs`, keeping screen and routing files under the 500-line hard limit.

Hardening status on 2026-05-16: save serialization now has focused tests covering Phase 6 bred egg inheritance and Phase 7 town job assignments, plus compatibility for older saves missing those fields. `screens::town` was split so town input/backdrop stays in `town.rs` and panel rendering lives in `town_panels.rs`; all Rust source files remain below the 500-line hard limit. The recurring browser console message `glBindTexture called with an already deleted texture ID 17` was investigated: Hatchspire does not create, load, draw, or delete explicit `Texture2D` assets, and the WebGL bundle matches the `macroquad 0.4.14` crate bundle, so the issue is tracked as a Macroquad/miniquad WebGL runtime investigation rather than an app-level texture lifecycle bug.

System depth status on 2026-05-16: monsters now carry save-compatible condition state for expedition fatigue and short injuries. Combat victory, fleeing, and defeat apply strain; tired monsters take small combat stat penalties and reduced town job output; injured monsters need rest before party assignment or workshop jobs; sleep restores HP, reduces fatigue, and clears injury timers. Stable, workshop, and tower screens now surface readiness so the player can make recovery decisions instead of treating HP as the only long-term cost.

Breeding and art pipeline status on 2026-05-16: bred eggs now store lineage quality and a deterministic monster art profile built from inherited species, element, temperament, passive, town skill, and seed data. Hatching applies lineage quality as a small bond/stat bonus and refreshes the child's art profile after inherited traits resolve. Local ComfyUI integration is available through `tools/export_monster_art_prompts.mjs` and `tools/generate_monster_art.ps1`, using the `H:\VideoGeneration` ComfyUI scripts against `127.0.0.1:8188`; a sample Slime/Rillfin lineage image is generated under `assets/generated/monster_art/`.

Before any shareable milestone:

- No compiler errors.
- No avoidable warnings.
- Save/load manually verified.
- Native launch verified.
- WebGL build verified through `publish.ps1`.

## Post-MVP Backlog

These items are deliberately out of the compact MVP unless a later pass chooses to pull them forward.

- Town navigation: keep the MVP as a screen-based hub; evaluate a walkable camp map only after the full loop is stable and worth expanding.
- Tower presence: keep the MVP as command-based room and combat screens; consider a player avatar only if exploration becomes spatial.
- Monster injury model: keep the current short-rest injury and fatigue model; evaluate deeper injury traits only after combat balance has more playthrough data.
- Unity reinterpretation: import only concepts that improve the Rust data loop; avoid porting old Unity scene/controller structure directly.
- Title and branding: keep `Hatchspire` as the player-facing title while the crate/package remains `monstron`.
- WebGL runtime log: reproduce `glBindTexture called with an already deleted texture ID 17` in a minimal Macroquad WebGL sample, then decide whether to upgrade Macroquad/miniquad, patch the bundle, or leave it documented if harmless.
- Polish pass: rebalance building costs, shop trades, job outputs, egg timers, and early tower rewards after several full-loop browser playthroughs.
- Visual QA: capture browser screenshots for town, hatchery, stable, breeding grove, workshop, shop, tower, and combat after the next UI polish pass.
- Art curation: review ComfyUI monster outputs, keep only approved images, then add a small runtime asset manifest once the visual style is stable.
