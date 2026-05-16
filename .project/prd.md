# Hatchspire - Product Requirements

> Working title for the `monstron` Rust project.
> This is a reinterpretation of the old Unity prototype, not a direct port.

## Product Summary

Hatchspire is a cozy monster-raising town-builder RPG built in Rust with Macroquad. The player discovers a ruined tower, hatches a weak starter slime, and grows a settlement around the tower by sending monsters inside for eggs, materials, relics, and story progress.

The design spine is:

```text
Town building -> Monster raising -> Dungeon depth -> Town building
```

The game succeeds when each day gives the player a meaningful reason to care about the town, the monsters, and the tower at the same time.

## Core Pillars

### 1. Town Growth

The town starts as a camp beside a broken tower and grows into a monster-focused settlement. Buildings are functional systems, not just decorations.

MVP buildings:

| Building | Purpose |
| --- | --- |
| Player Tent / House | Sleep, save, daily summary |
| Hatchery | Hatch and inspect eggs |
| Stable | Store and manage monsters |
| Workshop | Build and upgrade town structures |
| Shop | Buy basic supplies and sell surplus |

Post-MVP buildings include blacksmith, alchemy lab, breeding grove, tavern, shrine, library, farm, arena, and festival square.

### 2. Monster Raising

Monsters are adventurers and citizens. A monster should matter in combat, exploration, town work, breeding, and story flavor.

MVP monster properties:

| Property | Example |
| --- | --- |
| Species | Slime, Rootling, Glowmoth |
| Element | Water, Fire, Earth |
| Temperament | Cowardly, Loyal, Curious |
| Role | Tank, scout, striker, support |
| Passive | Finds herbs, resists poison, detects eggs |
| Town Skill | Farming, mining, guarding, hatching |

The starter slime remains narratively and mechanically relevant through unique bond progression and special town/dungeon utility.

### 3. Tower Dungeon

The tower is the main source of eggs, resources, relics, and enemy encounters. MVP scope is 10 floors with simple room navigation, loot, turn-based battles, and one boss.

MVP tower zones:

| Floors | Theme | Rewards |
| --- | --- | --- |
| 1-3 | Mossy Ruins | Wood, herbs, common eggs |
| 4-6 | Crystal Cracks | stone, crystal, patterned eggs |
| 7-10 | Sunken Garden | ore, rare herbs, first boss egg |

Future zones can extend upward into the spire and downward into sealed roots.

## MVP Scope

### Must Have

- Walkable or screen-based town hub.
- Daily cycle: wake, manage town, enter tower, return, sleep.
- Starter slime and small monster roster.
- Egg finding, egg hatching, and basic monster management.
- Five town buildings with upgrade hooks.
- Turn-based combat for 3 active monsters.
- 10-floor tower with enemies, loot, and one boss.
- Basic resources: wood, stone, ore, herbs, crystal, coins.
- Save/load for WebGL and native builds.
- Data-driven definitions in JSON.

### Should Have

- Three NPCs with services and friendship values.
- Simple gifts or request turn-ins.
- Early breeding prototype using inherited species, element, temperament, and passive traits.
- Auto-expedition prototype for cleared floors.
- Town activity log and monster event log.

### Not MVP

- Full romance arcs.
- Festivals.
- Large procedural town placement.
- AI-generated monster art pipeline.
- Deep multi-generation breeding.
- Advanced equipment and crafting.
- Full story acts and endings.

## Player Experience

The first playable milestone should communicate this arc:

1. The player arrives at a ruined tower camp.
2. A weak starter slime joins them.
3. The slime helps recover the first egg and basic materials.
4. The player builds the hatchery.
5. The egg hatches into a second monster.
6. A small team reaches floor 10 and defeats the first boss.

The emotional goal is that the player feels the town exists because the monsters are helping build it, not because buildings are abstract menu upgrades.

## Design Constraints

- The old Unity project is inspiration only. Existing names, mechanics, or data should be reused only when they fit the new design.
- Build for both desktop and WebGL from the beginning.
- Keep systems data-driven so monster species, eggs, buildings, NPCs, and floors can be expanded without rewriting engine logic.
- Prefer clear deterministic simulation over hidden randomness. Randomness should be inspectable and explainable where possible.
- UI actions should return intent/action objects. State mutation should live in game engines or state reducers.

## Success Criteria

The MVP is complete when:

- A new player can complete a 10-floor loop from fresh save to first boss.
- The player has hatched at least one monster from an egg.
- Building upgrades consume dungeon resources and unlock stronger progression.
- Combat supports a 3-front, 3-back party model internally, even if MVP battles begin with fewer monsters.
- Native and WebGL builds run without warnings.
- Save/load preserves town, monsters, eggs, resources, dungeon progress, and day count.
