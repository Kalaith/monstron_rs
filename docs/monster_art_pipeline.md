# Hatchspire Monster Art Pipeline

## Purpose

Breeding and hatching now create deterministic art DNA for each monster. The game keeps using procedural badges at runtime, while curated monster images are generated offline through local ComfyUI and can be reviewed before any asset is promoted into the game.

## Runtime Data

Each monster has a save-compatible `art_profile`:

- `species_hint`
- `silhouette`
- `palette`
- `markings`
- `accessory`
- `mood`
- `lineage_code`

Bred eggs also store lineage quality and a blended art profile. When the egg hatches, lineage quality gives the child a small bond/stat bonus and the final monster profile is refreshed from the inherited traits.

## Export Prompts

Generate the project prompt manifest:

```powershell
node tools\export_monster_art_prompts.mjs
```

This writes:

```text
assets/generated/monster_art/monster_art_prompts.json
```

The manifest includes one prompt per base species plus a sample Slime/Rillfin bred lineage.

## Generate With Local ComfyUI

ComfyUI is expected at:

```text
http://127.0.0.1:8188
```

Generate the sample bred lineage:

```powershell
.\tools\generate_monster_art.ps1 -Id sample_slime_rillfin_lineage
```

Generate every prompt in the manifest:

```powershell
.\tools\generate_monster_art.ps1 -All
```

Dry-run prompt output without generating:

```powershell
.\tools\generate_monster_art.ps1 -Test -Id sample_slime_rillfin_lineage
```

Generated images are saved under:

```text
assets/generated/monster_art/
```

## Current Defaults

- Script source: `H:\VideoGeneration\image_tools\art_pipeline\generate-image.ps1`
- ComfyUI server: `127.0.0.1:8188`
- Model: `plantMilkModelSuite_walnut.safetensors`
- Size: `768x768`
- Steps: `24`
- CFG: `6.5`
- Sampler: `dpmpp_2m`
- Scheduler: `karras`

The first ComfyUI test with `z-anime-base-aio-fp8.safetensors` produced an effectively blank image. `plantMilkModelSuite_walnut.safetensors` produced the more useful game-art sample, so it is the default for now.
