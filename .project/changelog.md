# Hatchspire - Changelog

All notable planning and implementation changes for the `monstron` Rust project will be documented here.

## [Unreleased]

### Added

- Scaffolded `monstron` as a RustGames workspace crate.
- Added Macroquad app entry point, `Game` owner, and explicit screen state machine.
- Added embedded JSON definitions for resources, five MVP buildings, six monster species, 12 egg types, and three NPCs.
- Added serializable `GameState` with resources, town, monster roster, egg inventory, tower progress, NPC relationships, story flags, and activity log.
- Added title, town, dungeon prep placeholder, tower placeholder, combat placeholder, and end-of-day screens.
- Added starter save/new game flow, cross-platform save/load through the shared persistence helper, and a starter slime.
- Updated `index.html` to load `monstron.wasm` and provide the WebGL localStorage bridge.
- Added Phase 2 town mechanics: build/upgrade spending, town rank, temporary scavenging, NPC greetings/friendship, and shop trades.
- Added Phase 3 monster and egg systems: typed traits, hatchery inventory, egg warming, deterministic hatching, stable roster view, and six-slot party management.
- Added deterministic egg placeholder visuals.
- Added Phase 4 tower exploration: 10 floor definitions, run pressure, loot and egg cargo, floor event generation, best-floor/unlock tracking, and return-to-town loot summaries.
- Replaced placeholder project documentation with a Hatchspire-focused PRD.
- Added Rust/Macroquad technical plan.
- Added phased build plan for the Rust conversion.
- Added dedicated Rust conversion plan under `docs/rust_conversion_plan.md`.

### Changed

- Defined the Unity project as inspiration only, not a one-to-one port target.
- Split persistent state into focused resource, town, monster, egg, tower, activity log, and root save-state modules.
- Tightened Rust file-size guidance to a 500-line hard limit and documented that UI belongs in `screens/` or `ui/`.

### Fixed

- Build gates now pass for `cargo check`, native dev build, and wasm release build.
- Updated the publish script to resolve the workspace target directory before packaging.
- Fixed local browser canvas sizing with inline host-page layout CSS and a fixed 1280x720 virtual camera.

## Milestones

| Version | Milestone | Target |
| --- | --- | --- |
| 0.1.0 | Rust foundation and empty playable shell | Phase 0 |
| 0.2.0 | Daily town loop and save/load | Phase 1 |
| 0.3.0 | Town, eggs, hatching, and tower exploration | Phases 2-4 |
| 0.4.0 | Turn-based combat MVP | Phase 5 |
| 0.5.0 | Breeding and first shareable MVP | Phases 6-7 |

Last updated: 2026-05-16
