# Hatchspire - Build Plan

> Project path: `H:\WebHatchery\RustGames\monstron`
> Current state: Phase 0 through Phase 4 complete.

## Current Decision

The old Unity project will be treated as concept reference only. The Rust game should borrow useful ideas such as monsters, eggs, dungeon runs, generated monster visuals, and town progression, but it should not attempt a file-by-file or system-by-system port.

## Phase 0 - Rust Foundation

Goal: turn the template folder into a buildable RustGames workspace member.

- [x] Create `Cargo.toml` for `monstron`.
- [x] Add `monstron` to `H:\WebHatchery\RustGames\Cargo.toml` workspace members.
- [x] Create `src/main.rs` and `src/game.rs`.
- [x] Create module directories for `data`, `state`, `engine`, `screens`, `ui`, `save`, and `assets`.
- [x] Update `index.html` to load the actual `monstron.wasm` output.
- [x] Confirm native `cargo check -p monstron` passes.
- [x] Confirm WebGL target can build.

Deliverable: empty but running application with main menu, placeholder town screen, and clean builds.

## Phase 1 - Vertical Slice Shell

Goal: establish the playable loop without full systems depth.

- [x] Implement `GameState` with day count, resources, tower progress, monsters, eggs, and log.
- [x] Add embedded JSON loader and initial data files.
- [x] Add screen state machine: main menu, town, dungeon prep, tower, combat placeholder, end-of-day.
- [x] Add starter save/new game flow.
- [x] Add activity log panel.
- [x] Add starter slime data.

Deliverable: player can start a new game, see town state, advance a day, and save/load.

## Phase 2 - Town MVP

Goal: make the town into the hub that consumes tower rewards.

- [x] Implement five MVP buildings: house, hatchery, stable, workshop, shop.
- [x] Add building costs and upgrade levels.
- [x] Add resource inventory and spending validation.
- [x] Add shop buy/sell actions.
- [x] Add three NPC service stubs with friendship values.
- [x] Add town rank calculation.

Deliverable: resources from test actions can build and upgrade the town, unlocking visible systems.

Note: `Scavenge` remains a convenience resource action, but Phase 4 tower loot now feeds the town economy.

## Phase 3 - Monster And Egg MVP

Goal: make eggs and monsters the emotional and mechanical center.

- [x] Add six monster species.
- [x] Add three elements.
- [x] Add temperament, role, passive, and town skill enums.
- [x] Add 12 egg definitions with rarity and possible hatch results.
- [x] Implement egg inventory.
- [x] Implement hatching rules and hatchery UI.
- [x] Implement stable roster UI.
- [x] Add deterministic generated pixel sprites or colored placeholders.

Deliverable: player can find eggs, hatch monsters, inspect traits, and manage a small roster.

Note: `Recover Egg` remains a hatchery-side fallback action. Phase 4 tower exploration now provides real floor egg rewards.

## Phase 4 - Tower Exploration MVP

Goal: build the 10-floor dungeon loop.

- [x] Implement tower floor data.
- [x] Add simple room or node exploration.
- [x] Add floor events: loot, enemy, egg, exit, boss.
- [x] Add inventory limits or return pressure.
- [x] Add floor unlock and best-depth tracking.
- [x] Add loot summary on return.

Deliverable: player can enter the tower, collect materials and eggs, return to town, and push deeper over time.

Note: enemy and boss events intentionally use pressure-only placeholder resolution until Phase 5 adds turn-based combat.

## Phase 5 - Turn-Based Combat MVP

Goal: ship the first real combat system around the 3-front, 3-back party structure.

- [ ] Implement combat slots: three front, three back.
- [ ] Implement enemy formation slots.
- [ ] Add combat stats: HP, attack, defense, speed, morale.
- [ ] Add action selection: attack, skill, defend, item, flee.
- [ ] Add basic target rules where front slots shield back slots.
- [ ] Add eight enemy types and one floor-10 boss.
- [ ] Add combat rewards and monster XP.
- [ ] Add defeat and rescue/return behavior.

Deliverable: player can fight through the first 10 floors and defeat the MVP boss.

## Phase 6 - Breeding And Progression

Goal: prove the longer-term monster growth loop.

- [ ] Add breeding grove unlock.
- [ ] Implement parent compatibility.
- [ ] Implement inheritance for species family, element, temperament, passive, and palette.
- [ ] Add child egg generation.
- [ ] Add simple mutation chance from tower floor origin.
- [ ] Add original slime bond progression hooks.

Deliverable: player can create a second-generation monster that is meaningfully shaped by both parents.

## Phase 7 - Polish And Publish

Goal: make the MVP coherent enough to share.

- [ ] Add title screen presentation.
- [ ] Add audio and basic settings.
- [ ] Add tooltips and clearer error messaging.
- [ ] Balance first 10 floors and early building costs.
- [ ] Verify save/load across native and WebGL.
- [ ] Run `cargo fmt`.
- [ ] Run `cargo clippy` or at minimum confirm no compiler warnings.
- [ ] Run `.\publish.ps1`.

Deliverable: playable WebGL/native MVP with the core loop intact.

## Implementation Priorities

1. Keep the data model stable before adding many content entries.
2. Build the full daily loop early, even with placeholder content.
3. Support the six-monster combat formation structurally from the start.
4. Use generated placeholder visuals until systems are proven.
5. Add AI-assisted monster art only after the inheritance model is stable.

## Build Status

Last verified on 2026-05-16:

- `cargo fmt -p monstron`
- `cargo check -p monstron`
- `cargo build -p monstron`
- `cargo build -p monstron --target wasm32-unknown-unknown --release`
- `.\publish.ps1 -WebGLOnly -SkipBuild -DryRun`

Current workspace status: `cargo check -p monstron`, native build, wasm release build, and WebGL dry-run packaging pass.
