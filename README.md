# Hatchspire

Hatchspire is a monster-raising town RPG about rebuilding a camp beneath a ruined tower and preparing companions for deeper dungeon runs.

You begin with a loyal slime, a cold hatchery brazier, and just enough supplies to make the camp useful again. Each day is a loop of preparation, risk, recovery, and growth.

## Gameplay

- Rebuild camp facilities such as the hatchery, stable, workshop, shop, and breeding grove.
- Hatch, raise, rest, and organize tower-born monsters.
- Choose a party before entering the tower.
- Fight, gather, and retreat before injuries and fatigue spiral.
- Spend rewards on stronger buildings, better recovery, and deeper expeditions.

## Goal

Turn a fragile camp into a working monster haven that can support increasingly dangerous tower runs.

## Controls

- Enter: new save or enter tower.
- Space: sleep.
- WASD / arrows: move through the tower view.
- Minimap: tracks discovered tower terrain.
- D: tower preparation.
- H: hatchery.
- R: stable or return from tower.
- B: breeding grove.
- W: workshop jobs.
- T: shop trades.
- C: scavenge supplies.
- A/S/D: combat attack, skill, or defend.
- I/F: combat herbs or flee.
- Esc: camp menu.
- S/L/T: save, load, or title inside menu.
- Mouse: build, open, trade, and greet.

## Current Scope

Playable camp and tower loop with monsters, hatching, breeding, jobs, expeditions, combat, fatigue, injuries, and recovery.

## Design

The design spine is a single circle:

```text
Town building -> Monster raising -> Tower depth -> Town building
```

Three pillars carry it:

- **Town growth.** The camp starts broken beside the tower and grows into a monster-focused settlement. Buildings are functional systems (hatchery, stable, workshop, shop, breeding grove), not decorations.
- **Monster raising.** A monster is an adventurer *and* a citizen, and should matter in combat, exploration, town work, breeding, and flavour. Each carries a species, element, temperament, role, passive, and town skill. The starter slime stays relevant through bond progression and unique utility.
- **Tower dungeon.** Ten floors across Mossy Ruins (1-3), Crystal Cracks (4-6), and Sunken Garden (7-10), ending at the Verdant Crown boss. The tower is the source of eggs, materials, relics, and encounters.

The emotional goal is that the town exists *because* the monsters are helping build it, rather than buildings being abstract menu upgrades.

Standing design constraints:

- Build for native and WebGL from the start.
- Keep systems data-driven so species, eggs, buildings, NPCs, and floors expand without engine rewrites.
- Prefer deterministic, inspectable simulation over hidden randomness.
- UI returns intent/action objects; state mutation lives in engines and reducers.
- The old Unity Monstron prototype is inspiration only, never a port target. `Hatchspire` is the player-facing title; the crate stays `monstron`.

## Documentation

- `docs/monster_art_pipeline.md` — art DNA, prompt export, and the local ComfyUI generation workflow.
- `TODO.md` — open polish, balance, and art-curation work.

