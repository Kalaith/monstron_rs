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
| Town | [`town_landmark_reference_v1.png`](../assets/generated/town/town_landmark_reference_v1.png) | Hatchery, stable, workshop, shop, grove, shrine, supplies, gate, notice board |
| Town atlas | [`town_sprite_atlas_v1.png`](../assets/generated/town/town_sprite_atlas_v1.png) | Six cleaned RGBA facility candidates for in-world interaction points |
| Dungeon | [`dungeon_landmark_reference_v1.png`](../assets/generated/dungeon/dungeon_landmark_reference_v1.png) | Egg nest, cache, stairs, enemy, crystal shrine, spring, machinery, gate, boss silhouette |
| Dungeon atlas | [`dungeon_sprite_atlas_v1.png`](../assets/generated/dungeon/dungeon_sprite_atlas_v1.png) | Six cleaned RGBA landmark candidates: egg nest, relic chest, stairs, crystal shrine, healing spring, tower machinery |
| Combat VFX atlas | [`combat_vfx_atlas_v1.png`](../assets/generated/combat/combat_vfx_atlas_v1.png) | Six cleaned RGBA overlays: ember burst, water splash, root snare, crystal shield, healing glow, stun stars |
| UI | [`living_tower_dungeon_mockup_v1.png`](ui_reference/living_tower_dungeon_mockup_v1.png) | World-first exploration screen and edge-mounted information hierarchy |

## Implementation notes

The generated sheets should be treated as visual references until individual
sprites are extracted, cleaned, and wired into the runtime. Preserve the
silhouette and palette relationships when producing final transparent assets.
Avoid adding permanent information panels that compete with the world layer.
