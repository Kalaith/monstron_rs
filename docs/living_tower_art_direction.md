# Living Tower Art Direction

These references establish the visual target for the Hatchspire sprite pass.
They are concept sheets and UI studies, not final runtime atlases.

## Direction

- Let the dungeon or camp occupy most of the canvas; UI belongs at the edges.
- Use dark charcoal stone, deep forest greens, moss, parchment-grey masonry, and warm amber lamps.
- Make monsters the brightest visual candy: chunky silhouettes, expressive faces, and pastel elemental accents.
- Communicate gameplay through illustrated objects: nests, caches, shrines, stairs, gates, and camp facilities.
- Keep fog-of-war nearly black for unknown space, silhouettes for discovered space, and full detail for explored space.

## Current reference sheets

| Area | Reference | Intended use |
| --- | --- | --- |
| Monsters | [`roster_reference_living_tower_v1.png`](../assets/generated/monster_art/roster_reference_living_tower_v1.png) | Roster silhouettes, elemental palette, size hierarchy |
| Monster atlas | [`monster_sprite_atlas_v1.png`](../assets/generated/monster_art/monster_sprite_atlas_v1.png) | Six cleaned RGBA sprite candidates: slimes, ember pup, stone pup, lantern moth |
| Battle monsters | [`battle_monster_atlas_v1.png`](../assets/generated/monster_art/battle_monster_atlas_v1.png) | Six larger battle-ready candidates: moss mite, rootling guardian, rillfin, glowmoth, quarryback, tower warden |
| Monster eggs | [`monster_egg_atlas_v1.png`](../assets/generated/monster_art/monster_egg_atlas_v1.png) | Six cleaned RGBA lifecycle objects: ember, rillfin, rootling, glowmoth, quarryback, and warden eggs |
| Monster reactions | [`monster_reaction_atlas_v1.png`](../assets/generated/monster_art/monster_reaction_atlas_v1.png) | Six cleaned RGBA expressive poses: protect, celebrate, frightened, injured, charge, and cast |
| Monster status icons | [`monster_status_icon_atlas_v1.png`](../assets/generated/monster_art/monster_status_icon_atlas_v1.png) | Six cleaned RGBA compact markers: curious, brave, tired, burning, protected, and rooted |
| Town | [`town_landmark_reference_v1.png`](../assets/generated/town/town_landmark_reference_v1.png) | Hatchery, stable, workshop, shop, grove, shrine, supplies, gate, notice board |
| Town atlas | [`town_sprite_atlas_v1.png`](../assets/generated/town/town_sprite_atlas_v1.png) | Six cleaned RGBA facility candidates for in-world interaction points |
| Town props | [`town_prop_atlas_v1.png`](../assets/generated/town/town_prop_atlas_v1.png) | Six cleaned RGBA interaction props: food trough, incubator, training ring, map table, herb rack, message perch |
| Town inhabitants | [`town_inhabitant_atlas_v1.png`](../assets/generated/town/town_inhabitant_atlas_v1.png) | Six cleaned RGBA inhabitants and helpers for a lived-in hub: caretaker, mapmaker, slime, courier, pup, root sage |
| Town wayfinding | [`town_wayfinding_atlas_v1.png`](../assets/generated/town/town_wayfinding_atlas_v1.png) | Six cleaned RGBA navigation landmarks: signpost, bell, milestone, bulletin board, bridge arch, return beacon |
| Dungeon | [`dungeon_landmark_reference_v1.png`](../assets/generated/dungeon/dungeon_landmark_reference_v1.png) | Egg nest, cache, stairs, enemy, crystal shrine, spring, machinery, gate, boss silhouette |
| Dungeon atlas | [`dungeon_sprite_atlas_v1.png`](../assets/generated/dungeon/dungeon_sprite_atlas_v1.png) | Six cleaned RGBA landmark candidates: egg nest, relic chest, stairs, crystal shrine, healing spring, tower machinery |
| Dungeon dressing | [`dungeon_dressing_atlas_v1.png`](../assets/generated/dungeon/dungeon_dressing_atlas_v1.png) | Six cleaned RGBA environment clusters: rubble arch, puddle floor, statue, root curtain, broken bridge, boss arena |
| Dungeon hazards | [`dungeon_hazard_atlas_v1.png`](../assets/generated/dungeon/dungeon_hazard_atlas_v1.png) | Six cleaned RGBA danger objects: poison bog, falling stones, flame jet, rune gear, thorn snare, void portal |
| Dungeon room modules | [`dungeon_room_module_atlas_v1.png`](../assets/generated/dungeon/dungeon_room_module_atlas_v1.png) | Six cleaned RGBA room vignettes: safe camp, treasure alcove, enemy room, egg chamber, stair landing, shrine room |
| Combat VFX atlas | [`combat_vfx_atlas_v1.png`](../assets/generated/combat/combat_vfx_atlas_v1.png) | Six cleaned RGBA overlays: ember burst, water splash, root snare, crystal shield, healing glow, stun stars |
| Exploration UI | [`living_tower_dungeon_mockup_v1.png`](ui_reference/living_tower_dungeon_mockup_v1.png) | World-first exploration screen and edge-mounted information hierarchy |
| Fog-map UI | [`living_tower_fog_map_mockup_v1.png`](ui_reference/living_tower_fog_map_mockup_v1.png) | Explored rooms, discovered silhouettes, fog-of-war, party cluster, and contextual stairs drawer |
| Battle UI | [`living_tower_battle_mockup_v1.png`](ui_reference/living_tower_battle_mockup_v1.png) | Battle diorama with immediate tactical information and a restrained command layer |
| Camp UI | [`living_tower_camp_mockup_v1.png`](ui_reference/living_tower_camp_mockup_v1.png) | World-first hatchery/camp screen with an egg drawer, resting party, resources, and touch actions |

## Implementation notes

The generated sheets should be treated as visual references until individual
sprites are extracted, cleaned, and wired into the runtime. Preserve the
silhouette and palette relationships when producing final transparent assets.
Avoid adding permanent information panels that compete with the world layer.
