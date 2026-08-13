# Dungeon Mockup Alignment Implementation Plan

Status: implemented through the full-screen Moss Gate acceptance pass; continue extending the same data-driven renderer to deeper floors.

The target is the world-first dungeon composition represented by `docs/ui_reference/living_tower_dungeon_mockup_v1.png`: a connected illustrated dungeon occupies the canvas, while a compact top bar, party rail, context drawer, expedition journal, and bottom action dock float over the world.

## Phase completion record

- Atlas registry: complete in `src/assets/mod.rs`; transparent room, fog, atmosphere, portrait, party, landmark, enemy, egg, cache and traversal atlases are addressable by typed helpers.
- Visual room graph: complete for the current run model; generated rooms are rendered as room-scale illustrated modules with purpose-aware scene selection.
- World camera/layered renderer: complete for the full-screen run view; room modules, connectors, entities, atmosphere, fog and overlays render in layers.
- Fog and lighting: complete for the acceptance pass using fog silhouettes, discovered tinting, landmark glows and biome atmosphere overlays.
- Physical landmarks: complete for cache, egg, enemy, boss, stairs and exit scene variants.
- Mockup HUD: complete with top run overlay, party rail, right context drawer, expedition journal, bottom action dock and persistent touch controls.
- Room-level interaction: complete as a visible-world tap-to-step affordance with directional touch fallback; tile movement remains authoritative.
- Data-driven biome variants: complete for room-family selection by floor group; deeper floor scene families and richer hazards/recovery/boss compositions remain extension points.

## Phases

1. **Atlas registry and composition proof**
   - Add typed dungeon visual identifiers and atlas metadata.
   - Use transparent runtime atlases only.
   - Define cells, anchors, visual bounds, and layer order.
   - Capture a Moss Gate composition proof.

2. **Visual room graph**
   - Preserve `TowerMapState` as gameplay authority.
   - Derive room-purpose, biome, connector, landmark, and world-position presentation records.
   - Render rooms as connected illustrated modules instead of exposed square debug tiles.

3. **World camera and layered renderer**
   - Add a camera centered on the party.
   - Render background, discovered silhouettes, room modules, connectors, dressing, landmarks, entities, lights, and fog in layers.
   - Keep deterministic placement from the run seed.

4. **Fog and lighting**
   - Use unknown, discovered, and visible visual states.
   - Apply generated fog/silhouette and atmosphere overlays.
   - Add party, shrine, enemy, and boss light pools.

5. **Physical landmarks**
   - Convert egg, loot, enemy, boss, stairs, and exit objects into environmental scenes.
   - Keep small markers only as clarity accents.

6. **Mockup HUD**
   - Add top run bar, party rail, context drawer, expedition journal, and bottom action dock.
   - Keep controls touch-sized and explicit.
   - Retain directional movement as a fallback.

7. **Room-level interaction**
   - Add tap targets for adjacent rooms and landmarks.
   - Keep tile movement authoritative and available.
   - Surface Explore, Camp, Retreat, and contextual actions.

8. **Data-driven biome variants**
   - Map floors to moss, flooded, ember, frost, root, and void room families.
   - Add hazard, recovery, boss, and reward compositions.

9. **Verification**
   - Capture the tower state and replace `docs/verification/ui_tower.png`.
   - Run Rust tests and `publish.ps1`.
   - Commit each coherent milestone.

## Acceptance

The primary capture must show a full-canvas connected dungeon with room-scale art, soft fog, local lighting, physical landmarks, readable party identity, and a compact touch-first HUD. No magenta chroma assets or square tile grid may be visible in the final world presentation.
