# TODO — Hatchspire

## Polish

- Add audio and a basic settings surface.
- Add tooltips and clearer error messaging across the facility screens.
- Give the title screen a real presentation pass.
- Verify save/load end to end on both native and WebGL after each save-schema change.

## Balance

- Rebalance building costs, shop trades, job outputs, egg timers, and early tower rewards after several full-loop browser playthroughs.
- Add seeded combat replays so balance issues reproduce and the tower difficulty ramp can be verified.

## Art

- Curate the ComfyUI monster outputs under `assets/generated/monster_art/`, keep only approved images, and add a small runtime asset manifest once the visual style settles.
- Capture browser screenshots for town, hatchery, stable, breeding grove, workshop, shop, tower, and combat after the next UI polish pass.

## Engineering

- Add integration tests for the hatchery, shop, stable, tower, combat, and town purchase/progression flows.
- Centralise monster stat curves, cooldowns, shop inventory, and tower rewards into shared config fixtures.
- Split combat and town screen mutation from rendering so progression changes flow through explicit commands.
- Reproduce the WebGL `glBindTexture called with an already deleted texture ID 17` message in a minimal Macroquad sample, then decide whether to upgrade macroquad/miniquad, patch the bundle, or document it as harmless.
