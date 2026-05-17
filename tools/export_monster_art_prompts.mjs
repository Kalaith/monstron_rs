import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";

const outPath = resolve(
  process.argv[2] ?? "assets/generated/monster_art/monster_art_prompts.json",
);

const speciesData = JSON.parse(
  await readFile("assets/data/monster_species.json", "utf8"),
).monster_species;

const prompts = [
  ...speciesData.map((species, index) =>
    promptEntry({
      id: species.id,
      filename: species.id,
      group: "species",
      species,
      seed: 0x5000 + index * 97,
      lineage: `${species.name} baseline`,
      quality: "steady",
    }),
  ),
  promptEntry({
    id: "sample_slime_rillfin_lineage",
    filename: "sample_slime_rillfin_lineage",
    group: "bred_lineage",
    species: speciesData.find((species) => species.id === "slime"),
    seed: 0x51a15eed ^ 0xbee57001,
    lineage: "Slime and Rillfin bred water-support lineage",
    quality: "resonant",
    overrides: {
      palette: "glassy aqua, seafoam, pearl white, and pale violet palette",
      markings: "coin-like freckles mixed with soft healer crescent highlights",
      accessory: "small hatchery charm and egg-warming sash",
      mood: "loyal but gentle expression, companionable and calm",
    },
  }),
  promptEntry({
    id: "attempt_slime_rootling_mossy_lineage",
    filename: "attempt_slime_rootling_mossy_lineage",
    group: "bred_lineage_attempt",
    species: speciesData.find((species) => species.id === "slime"),
    seed: 0x51a15eed ^ 0x7007,
    lineage: "Slime and Rootling mossy egg-care scout lineage",
    quality: "steady",
    overrides: {
      palette: "glassy teal, moss green, pale stone, and pearl palette",
      markings: "coin-like freckles mixed with protective leaf-vein patches",
      accessory: "small hatchery charm wrapped in sprout twine",
      mood: "loyal and patient expression, low watchful stance",
    },
  }),
  promptEntry({
    id: "attempt_rootling_pebblepup_quarry_lineage",
    filename: "attempt_rootling_pebblepup_quarry_lineage",
    group: "bred_lineage_attempt",
    species: speciesData.find((species) => species.id === "rootling"),
    seed: 0x7107_1eaf ^ 0x9ebb_1e01,
    lineage: "Rootling and Pebblepup quarry-guard lineage",
    quality: "resonant",
    overrides: {
      palette: "fern green, mineral gray, bark brown, and amber palette",
      markings: "leaf-vein plates mixed with pebble spots and sturdy facets",
      accessory: "sprout charm and simple camp guard neckerchief",
      mood: "calm brave expression, squared sturdy posture",
    },
  }),
  promptEntry({
    id: "attempt_glowmoth_emberkit_lantern_lineage",
    filename: "attempt_glowmoth_emberkit_lantern_lineage",
    group: "bred_lineage_attempt",
    species: speciesData.find((species) => species.id === "glowmoth"),
    seed: 0x6100_6070 ^ 0xe4be_1a1,
    lineage: "Glowmoth and Emberkit lantern striker lineage",
    quality: "resonant",
    overrides: {
      palette: "lantern yellow, coral, rose red, smoke gray, and brass palette",
      markings: "soft glowing speckles mixed with warm ember streaks",
      accessory: "tiny lantern glow accents and little brass workshop tag",
      mood: "curious restless expression, bright alert wing pose",
    },
  }),
  promptEntry({
    id: "attempt_emberkit_pebblepup_ore_lineage",
    filename: "attempt_emberkit_pebblepup_ore_lineage",
    group: "bred_lineage_attempt",
    species: speciesData.find((species) => species.id === "emberkit"),
    seed: 0xe4be_1a1 ^ 0x9ebb_1e01,
    lineage: "Emberkit and Pebblepup ore-veined salvage lineage",
    quality: "potent",
    overrides: {
      palette: "charcoal, ember orange, mineral gray, amber, and warm gold palette",
      markings: "ember streaks mixed with mineral chips and glowing ore seams",
      accessory: "brass workshop tag and simple camp guard neckerchief",
      mood: "brave restless expression, ready-to-pounce stance",
    },
  }),
  promptEntry({
    id: "attempt_glowmoth_slime_moonlit_lineage",
    filename: "attempt_glowmoth_slime_moonlit_lineage",
    group: "bred_lineage_attempt",
    species: speciesData.find((species) => species.id === "glowmoth"),
    seed: 0x6100_6070 ^ 0x51a15eed,
    lineage: "Glowmoth and Slime moonlit egg-sense lineage",
    quality: "steady",
    overrides: {
      palette: "soft violet, glassy turquoise, pale mint, and warm lantern cream palette",
      markings: "nest-shaped glowing motifs mixed with tiny coin-like freckles",
      accessory: "tiny lantern glow accents and small hatchery charm",
      mood: "curious loyal expression, hovering companionable pose",
    },
  }),
  promptEntry({
    id: "attempt_rillfin_rootling_garden_lineage",
    filename: "attempt_rillfin_rootling_garden_lineage",
    group: "bred_lineage_attempt",
    species: speciesData.find((species) => species.id === "rillfin"),
    seed: 0xbee5_7001 ^ 0x7107_1eaf,
    lineage: "Rillfin and Rootling garden recovery lineage",
    quality: "resonant",
    overrides: {
      palette: "seafoam, fern green, pearl white, bark brown, and pale stone palette",
      markings: "soft crescent healer highlights mixed with protective leaf veins",
      accessory: "egg-warming sash with sprout charm and nest feathers",
      mood: "gentle patient expression, grounded healer posture",
    },
  }),
  promptEntry({
    id: "attempt_rillfin_pebblepup_sunken_lineage",
    filename: "attempt_rillfin_pebblepup_sunken_lineage",
    group: "bred_lineage_attempt",
    species: speciesData.find((species) => species.id === "rillfin"),
    seed: 0xbee5_7001 ^ 0x9ebb_1e01,
    lineage: "Rillfin and Pebblepup sunken stone-support lineage",
    quality: "steady",
    overrides: {
      palette: "deep blue, seafoam, mineral gray, pearl, and slate palette",
      markings: "healer crescent highlights mixed with pebble spots and facets",
      accessory: "egg-warming sash and simple camp guard neckerchief",
      mood: "gentle brave expression, compact protective stance",
    },
  }),
  promptEntry({
    id: "attempt_boss_rootling_glowmoth_crown_lineage",
    filename: "attempt_boss_rootling_glowmoth_crown_lineage",
    group: "bred_lineage_attempt",
    species: speciesData.find((species) => species.id === "rootling"),
    seed: 0xb055_e66 ^ 0x7107_1eaf ^ 0x6100_6070,
    lineage: "Rootling and Glowmoth verdant crown boss-egg lineage",
    quality: "mythic",
    overrides: {
      palette: "moss green, lantern gold, rose red, pale stone, and deep leaf palette",
      markings: "leaf-vein armor mixed with nest-shaped glowing motifs",
      accessory: "tiny lantern glow accents set into a leafy crown charm",
      mood: "calm curious expression, protective boss-born posture",
    },
  }),
  ...calibrationEntries(speciesData),
  ...strictSpriteCalibrationEntries(),
  ...leafpupSeedSweepEntries(),
  ...bipedLeafStoneSeedSweepEntries(),
  ...portraitLeafStoneSeedSweepEntries(),
  ...headOnlyLeafStoneSeedSweepEntries(),
  ...emberkitHeadSeedSweepEntries(),
];

const manifest = {
  generated_by: "tools/export_monster_art_prompts.mjs",
  source: "Hatchspire deterministic monster art profiles",
  comfyui_defaults: {
    server: "127.0.0.1:8188",
    model: "plantMilkModelSuite_walnut.safetensors",
    width: 768,
    height: 768,
    steps: 24,
    cfg: 6.5,
    sampler: "dpmpp_2m",
    scheduler: "karras",
  },
  image_prompts: prompts,
};

await mkdir(dirname(outPath), { recursive: true });
await writeFile(outPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
console.log(`Wrote ${prompts.length} monster art prompt(s) to ${outPath}`);

function promptEntry({ id, filename, group, species, seed, lineage, quality, overrides = {} }) {
  if (!species) {
    throw new Error(`Missing species for prompt ${id}`);
  }

  const profile = {
    species_hint: species.name,
    silhouette: silhouetteForSpecies(species.id),
    palette: paletteForElement(species.element, seed),
    markings: markingsForPassive(species.passive),
    accessory: accessoryForTownSkill(species.town_skill),
    mood: moodForTemperament(species.temperament),
    ...overrides,
  };

  return {
    id,
    filename,
    group,
    species_id: species.id,
    lineage,
    quality,
    art_profile: profile,
    Width: 768,
    Height: 768,
    Steps: 24,
    CFG: 6.5,
    Seed: seed & 0x7fffffff,
    Model: "plantMilkModelSuite_walnut.safetensors",
    Sampler: "dpmpp_2m",
    Scheduler: "karras",
    Prompt: buildPrompt(profile, lineage, quality),
    NegativePrompt:
      "blank image, empty image, whiteout, overexposed, low contrast, photorealistic, 3d render, toy figurine, sculpture, plastic material, human, humanoid, anime girl, multiple creatures, duplicate character, character lineup, evolution chart, comparison sheet, labeled sheet, labels, captions, detailed scenery, landscape, room, forest, cave, nest, pile of coins, egg, separate lantern, hanging lantern, pedestal, platform, base, ground patch, floor line, cast shadow, frame, ornate border, emblem, mandala, magic circle, icon badge, loose props, background objects, busy background, gradient background, UI, HUD, text, logo, watermark, cropped, blurry, noisy, extra limbs, harsh shadow, gore",
  };
}

function buildPrompt(profile, lineage, quality) {
  return [
    "one single friendly tower-born monster, 2D game creature sprite asset",
    "centered full body, compact readable silhouette, visible creature, high contrast, three-quarter view",
    "isolated character only on a perfectly plain flat warm gray background",
    "exactly one creature, not a character sheet, not a lineup, not an evolution chart",
    "no scenery, no environmental objects, no separate props, no floor objects, no base, no ground patch, no decorative frame",
    "any accessory must be physically worn by the monster or held in its paws",
    `${quality} ${lineage}`,
    profile.species_hint,
    profile.silhouette,
    profile.palette,
    profile.markings,
    profile.accessory,
    profile.mood,
    "flat 2D illustration, soft painterly game art, pixel-friendly shapes, clean outline, collectible monster RPG style, not a 3D render",
  ].join(", ");
}

function calibrationEntries(speciesData) {
  const byId = (id) => speciesData.find((species) => species.id === id);
  const common = {
    accessory: "tiny worn collar charm, no separate props",
  };
  return [
    promptEntry({
      id: "calibration_rootling_pebblepup_quarry_01",
      filename: "calibration_rootling_pebblepup_quarry_01",
      group: "prompt_calibration",
      species: byId("rootling"),
      seed: 0x701001,
      lineage: "Rootling and Pebblepup quarry guard lineage",
      quality: "resonant",
      overrides: {
        ...common,
        palette: "fern green, mineral gray, bark brown, ivory, and amber palette",
        markings: "leaf-vein plates mixed with pebble spots and sturdy facets",
        mood: "calm brave expression, squared sturdy posture",
      },
    }),
    promptEntry({
      id: "calibration_rootling_pebblepup_quarry_02",
      filename: "calibration_rootling_pebblepup_quarry_02",
      group: "prompt_calibration",
      species: byId("pebblepup"),
      seed: 0x701002,
      lineage: "Pebblepup and Rootling quarry guard lineage",
      quality: "resonant",
      overrides: {
        ...common,
        palette: "moss green, slate gray, clay, pale stone, and warm amber palette",
        markings: "mineral chips mixed with small leaf-vein markings",
        mood: "brave patient expression, sturdy compact stance",
      },
    }),
    promptEntry({
      id: "calibration_slime_rootling_mossy_01",
      filename: "calibration_slime_rootling_mossy_01",
      group: "prompt_calibration",
      species: byId("slime"),
      seed: 0x702001,
      lineage: "Slime and Rootling mossy egg-care lineage",
      quality: "steady",
      overrides: {
        ...common,
        palette: "glassy teal, moss green, pearl, and pale stone palette",
        markings: "soft freckles mixed with small leaf-vein markings on the body",
        mood: "loyal patient expression, soft low stance",
      },
    }),
    promptEntry({
      id: "calibration_slime_rootling_mossy_02",
      filename: "calibration_slime_rootling_mossy_02",
      group: "prompt_calibration",
      species: byId("rootling"),
      seed: 0x702002,
      lineage: "Rootling and Slime mossy scout lineage",
      quality: "steady",
      overrides: {
        ...common,
        palette: "seafoam, fern green, bark brown, and pearl palette",
        markings: "translucent dew spots mixed with leaf-vein patches",
        mood: "loyal grounded expression, watchful posture",
      },
    }),
    promptEntry({
      id: "calibration_glowmoth_slime_moonlit_01",
      filename: "calibration_glowmoth_slime_moonlit_01",
      group: "prompt_calibration",
      species: byId("glowmoth"),
      seed: 0x703001,
      lineage: "Glowmoth and Slime moonlit egg-sense lineage",
      quality: "steady",
      overrides: {
        ...common,
        palette: "soft violet, glassy turquoise, pale mint, and warm cream palette",
        markings: "small glowing body speckles mixed with subtle slime freckles",
        mood: "curious loyal expression, hovering friendly pose",
      },
    }),
    promptEntry({
      id: "calibration_glowmoth_slime_moonlit_02",
      filename: "calibration_glowmoth_slime_moonlit_02",
      group: "prompt_calibration",
      species: byId("slime"),
      seed: 0x703002,
      lineage: "Slime and Glowmoth moonlit scout lineage",
      quality: "steady",
      overrides: {
        ...common,
        palette: "rain blue, pale violet, mint glow, and pearl palette",
        markings: "tiny glowing speckles embedded inside the translucent body",
        mood: "loyal curious expression, gentle upward gaze",
      },
    }),
    promptEntry({
      id: "calibration_emberkit_pebblepup_ore_01",
      filename: "calibration_emberkit_pebblepup_ore_01",
      group: "prompt_calibration",
      species: byId("emberkit"),
      seed: 0x704001,
      lineage: "Emberkit and Pebblepup ore-veined salvage lineage",
      quality: "potent",
      overrides: {
        ...common,
        palette: "charcoal, ember orange, mineral gray, amber, and warm gold palette",
        markings: "ember streaks mixed with mineral chips on the body",
        mood: "brave restless expression, ready-to-pounce stance",
      },
    }),
    promptEntry({
      id: "calibration_emberkit_pebblepup_ore_02",
      filename: "calibration_emberkit_pebblepup_ore_02",
      group: "prompt_calibration",
      species: byId("pebblepup"),
      seed: 0x704002,
      lineage: "Pebblepup and Emberkit ore-veined striker lineage",
      quality: "potent",
      overrides: {
        ...common,
        palette: "slate gray, ember orange, brass gold, and charcoal palette",
        markings: "small warm cracks mixed with stone facets",
        mood: "brave lively expression, compact striker stance",
      },
    }),
    promptEntry({
      id: "calibration_rillfin_rootling_garden_01",
      filename: "calibration_rillfin_rootling_garden_01",
      group: "prompt_calibration",
      species: byId("rillfin"),
      seed: 0x705001,
      lineage: "Rillfin and Rootling garden recovery lineage",
      quality: "resonant",
      overrides: {
        ...common,
        palette: "seafoam, fern green, pearl white, bark brown, and pale stone palette",
        markings: "crescent healer highlights mixed with subtle leaf veins",
        mood: "gentle patient expression, grounded healer posture",
      },
    }),
    promptEntry({
    id: "calibration_rillfin_pebblepup_sunken_01",
      filename: "calibration_rillfin_pebblepup_sunken_01",
      group: "prompt_calibration",
      species: byId("rillfin"),
      seed: 0x706001,
      lineage: "Rillfin and Pebblepup sunken stone support lineage",
      quality: "steady",
      overrides: {
        ...common,
        palette: "deep blue, seafoam, mineral gray, pearl, and slate palette",
        markings: "small crescent highlights mixed with pebble spots",
        mood: "gentle brave expression, compact protective stance",
      },
    }),
    promptEntry({
      id: "calibration_rillfin_rootling_garden_strict_01",
      filename: "calibration_rillfin_rootling_garden_strict_01",
      group: "prompt_calibration",
      species: byId("rillfin"),
      seed: 0x707001,
      lineage: "Rillfin and Rootling garden recovery lineage",
      quality: "resonant",
      overrides: {
        ...common,
        palette: "seafoam, fern green, pearl white, bark brown, and pale stone palette",
        markings: "crescent healer highlights mixed with subtle leaf veins on one body",
        mood: "gentle patient expression, one grounded healer creature",
      },
    }),
    promptEntry({
      id: "calibration_rootling_pebblepup_quarry_strict_01",
      filename: "calibration_rootling_pebblepup_quarry_strict_01",
      group: "prompt_calibration",
      species: byId("rootling"),
      seed: 0x708001,
      lineage: "Rootling and Pebblepup quarry guard lineage",
      quality: "resonant",
      overrides: {
        ...common,
        palette: "moss green, slate gray, clay, pale stone, and warm amber palette",
        markings: "leaf-vein plates mixed with pebble spots on the body",
        mood: "calm brave expression, no base under the feet",
      },
    }),
    promptEntry({
      id: "calibration_slime_rootling_mossy_strict_01",
      filename: "calibration_slime_rootling_mossy_strict_01",
      group: "prompt_calibration",
      species: byId("slime"),
      seed: 0x709001,
      lineage: "Slime and Rootling mossy egg-care lineage",
      quality: "steady",
      overrides: {
        ...common,
        palette: "glassy teal, moss green, pearl white, and pale stone palette",
        markings: "small green leaf-like spots embedded inside the slime body",
        mood: "loyal patient expression, one simple soft creature",
      },
    }),
    promptEntry({
      id: "calibration_glowmoth_slime_moonlit_strict_01",
      filename: "calibration_glowmoth_slime_moonlit_strict_01",
      group: "prompt_calibration",
      species: byId("glowmoth"),
      seed: 0x70a001,
      lineage: "Glowmoth and Slime moonlit egg-sense lineage",
      quality: "steady",
      overrides: {
        ...common,
        palette: "soft violet, glassy turquoise, pale mint, and pearl palette",
        markings: "small glowing dots on the body and wings only",
        mood: "curious loyal expression, one hovering creature",
      },
    }),
  ];
}

function strictSpriteCalibrationEntries() {
  return [
    strictSpriteEntry({
      id: "sprite_calibration_quarry_leafpup_01",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, hybrid design blending a small leafy root creature with a compact stone puppy, leaf ears attached to the head, chunky pebble paws, gem nose, pebble spots on the body, tiny worn collar charm attached to the neck, fern green and slate gray palette, calm brave expression",
      seed: 0x810001,
    }),
    strictSpriteEntry({
      id: "sprite_calibration_quarry_leafpup_02",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, compact stone puppy body with sprout crest and small stump legs, mineral gray body with moss green patches, simple readable silhouette, no object beside it, no base under it, tiny collar tag attached to the neck, brave patient expression",
      seed: 0x810002,
    }),
    strictSpriteEntry({
      id: "sprite_calibration_mossy_slime_01",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, round translucent slime body with small leaf ears attached to the body, mossy green freckles embedded inside the slime, tiny feet, glassy teal and moss green palette, loyal patient expression, simple silhouette",
      seed: 0x810003,
    }),
    strictSpriteEntry({
      id: "sprite_calibration_mossy_slime_02",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, soft slime creature with a leafy crown growing from its head, no separate plants, no separate objects, aqua body with pale stone and fern markings, compact mascot proportions, gentle loyal expression",
      seed: 0x810004,
    }),
    strictSpriteEntry({
      id: "sprite_calibration_moonlit_moth_01",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, small moth sprite with rounded translucent slime belly, soft wings attached to the body, glowing dots on wings only, pale violet and mint turquoise palette, curious loyal expression, hovering pose without shadow",
      seed: 0x810005,
    }),
    strictSpriteEntry({
      id: "sprite_calibration_moonlit_moth_02",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, tiny winged slime mascot with two antennae and small moth wings attached, no second creature, no moon, no stars, pearl blue body with violet wing tips, simple clean outline, bright curious expression",
      seed: 0x810006,
    }),
    strictSpriteEntry({
      id: "sprite_calibration_ore_emberpup_01",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, small ember fox puppy with stone paws and mineral spots on the body, flame tail attached to the creature, charcoal and ember orange palette, no loose fire, no rocks around it, brave lively expression",
      seed: 0x810007,
    }),
    strictSpriteEntry({
      id: "sprite_calibration_ore_emberpup_02",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, compact stone puppy with ember ears and a small attached flame tail, slate gray body with warm orange cracks, clean outline, no ground patch, no props, restless brave expression",
      seed: 0x810008,
    }),
    strictSpriteEntry({
      id: "sprite_calibration_garden_rillroot_01",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, gentle finned amphibian with small leaf fins attached to the head and tail, seafoam body with fern green markings, soft crescent healer marks on the body, no extra creatures, no labels, patient healer expression",
      seed: 0x810009,
    }),
    strictSpriteEntry({
      id: "sprite_calibration_sunken_rillpup_01",
      prompt:
        "one original cute fantasy monster game sprite, isolated full body, centered, exactly one creature, gentle amphibian puppy hybrid with water fins attached and chunky pebble paws, deep blue and mineral gray palette, small crescent spots on the body, no stones around it, no props, protective brave expression",
      seed: 0x81000a,
    }),
  ];
}

function strictSpriteEntry({ id, prompt, seed }) {
  return {
    id,
    filename: id,
    group: "strict_sprite_calibration",
    species_id: "hybrid",
    lineage: "strict creature-only calibration",
    quality: "calibration",
    art_profile: {},
    Width: 768,
    Height: 768,
    Steps: 24,
    CFG: 6.5,
    Seed: seed & 0x7fffffff,
    Model: "plantMilkModelSuite_walnut.safetensors",
    Sampler: "dpmpp_2m",
    Scheduler: "karras",
    Prompt: [
      prompt,
      "plain uniform warm gray background",
      "no scenery, no environmental objects, no separate props, no floor, no base, no platform, no decorative frame",
      "flat 2D painterly game art, compact readable silhouette, high contrast, soft clean outline, collectible monster RPG asset",
    ].join(", "),
    NegativePrompt:
      "multiple creatures, two creatures, duplicate creature, character lineup, evolution chart, comparison sheet, labeled sheet, labels, captions, text, logo, watermark, human, humanoid, anime girl, photorealistic, 3d render, toy, sculpture, scenery, landscape, room, forest, cave, nest, egg, coins, lantern, separate object, loose prop, ground patch, platform, base, pedestal, floor line, cast shadow, ornate frame, border, emblem, mandala, magic circle, icon badge, busy background, gradient background, cropped, blurry, noisy, extra limbs",
  };
}

function leafpupSeedSweepEntries() {
  const prompt =
    "one cute compact leaf-eared stone puppy monster, original fantasy monster character, full body centered, exactly one creature only, leaf ears physically attached to the head, small sprout crest attached to the head, chunky pebble paws, gem nose, pebble spots embedded on the body, tiny collar charm attached to the neck, moss green and slate gray palette, calm brave expression, feet fully visible, floating slightly in empty space with no ground contact";
  return Array.from({ length: 10 }, (_, index) =>
    strictCreatureEntry({
      id: `sprite_leafpup_seed_${String(index + 1).padStart(2, "0")}`,
      prompt,
      seed: 0x820000 + index * 997,
    }),
  );
}

function bipedLeafStoneSeedSweepEntries() {
  const prompt =
    "one small biped leaf-stone monster, original fantasy monster character, full body centered, exactly one creature only, rounded friendly face, leafy crown physically attached to the head, short stump legs, small rounded arms, pebble spots embedded on the body, simple dark neckerchief worn around the neck, moss green and pale slate palette, calm brave expression, compact readable silhouette";
  return Array.from({ length: 10 }, (_, index) =>
    strictCreatureEntry({
      id: `sprite_leafstone_seed_${String(index + 1).padStart(2, "0")}`,
      prompt,
      seed: 0x830000 + index * 1231,
    }),
  );
}

function portraitLeafStoneSeedSweepEntries() {
  const prompt =
    "one cute leaf-stone monster portrait, original fantasy monster character, centered head and upper torso only, exactly one creature only, no legs, no feet, no ground, rounded friendly face, leafy crown physically attached to the head, small rounded arms partly visible, pebble spots embedded on cheeks and torso, simple dark neckerchief worn around the neck, moss green and pale slate palette, calm brave expression, compact readable silhouette";
  return Array.from({ length: 10 }, (_, index) =>
    strictCreatureEntry({
      id: `sprite_leafstone_portrait_seed_${String(index + 1).padStart(2, "0")}`,
      prompt,
      seed: 0x840000 + index * 1427,
    }),
  );
}

function headOnlyLeafStoneSeedSweepEntries() {
  const prompt =
    "one cute leaf-stone monster head portrait icon, original fantasy monster character, centered head only, exactly one creature only, no body, no torso, no arms, no legs, no feet, rounded friendly face, leafy crown physically attached to the head, small pebble spots embedded on cheeks, moss green and pale slate palette, calm brave expression, compact readable silhouette";
  return Array.from({ length: 40 }, (_, index) =>
    strictCreatureEntry({
      id: `sprite_leafstone_head_seed_${String(index + 1).padStart(2, "0")}`,
      prompt,
      seed: 0x850000 + index * 1613,
    }),
  );
}

function emberkitHeadSeedSweepEntries() {
  const prompt =
    "one cute Emberkit monster head portrait icon, original fantasy monster character based on a small ember fox kit, centered head and upper neck only, exactly one creature only, no body, no torso, no arms, no legs, no feet, alert fox ears physically attached to the head, small attached flame-shaped forehead tuft, warm ember cheek markings, charcoal and ember orange palette with warm gold accents, restless brave expression, compact readable silhouette";
  return Array.from({ length: 40 }, (_, index) =>
    strictCreatureEntry({
      id: `sprite_emberkit_head_seed_${String(index + 1).padStart(2, "0")}`,
      prompt,
      seed: 0x860000 + index * 1759,
      group: "emberkit_head_seed_sweep",
      speciesId: "emberkit",
      lineage: "emberkit prompt calibration",
    }),
  );
}

function strictCreatureEntry({
  id,
  prompt,
  seed,
  group = "leafpup_seed_sweep",
  speciesId = "rootling_pebblepup_hybrid",
  lineage = "rootling-pebblepup prompt calibration",
}) {
  return {
    id,
    filename: id,
    group,
    species_id: speciesId,
    lineage,
    quality: "calibration",
    art_profile: {},
    Width: 768,
    Height: 768,
    Steps: 26,
    CFG: 7.8,
    Seed: seed & 0x7fffffff,
    Model: "plantMilkModelSuite_walnut.safetensors",
    Sampler: "dpmpp_2m",
    Scheduler: "karras",
    Prompt: [
      prompt,
      "plain uniform warm gray background only",
      "no scenery, no floor, no shadow, no platform, no base, no pedestal, no separate objects, no loose leaves, no loose rocks, no symbols, no text",
      "flat 2D painterly creature game art, compact readable silhouette, high contrast, soft clean outline, collectible monster RPG style",
    ].join(", "),
    NegativePrompt:
      "multiple creatures, two creatures, duplicate creature, character lineup, evolution chart, comparison sheet, labels, captions, text, logo, watermark, detached leaves, loose leaf, loose rock, rock pile, ground patch, platform, stand, base, pedestal, floor, floor line, cast shadow, scenery, landscape, forest, cave, room, nest, egg, coins, lantern, separate object, loose prop, ornate frame, border, emblem, mandala, magic circle, icon badge, busy background, gradient background, human, humanoid, anime girl, photorealistic, 3d render, toy, sculpture, cropped, blurry, noisy, extra limbs",
  };
}

function silhouetteForSpecies(speciesId) {
  switch (speciesId) {
    case "slime":
      return "round translucent slime body, soft droplet ears, tiny feet";
    case "rootling":
      return "small root creature, leafy crown, sturdy stump legs";
    case "glowmoth":
      return "mothlike sprite, wide soft wings, glowing antennae";
    case "pebblepup":
      return "compact stone puppy, chunky paws, gem nose";
    case "emberkit":
      return "small ember fox kit, flame tail, alert ears";
    case "rillfin":
      return "gentle finned amphibian, flowing whiskers, water fins";
    default:
      return "small friendly tower-born monster, readable compact silhouette";
  }
}

function paletteForElement(element, seed) {
  const options = {
    Water: [
      "aqua, teal, pearl, and deep blue palette",
      "seafoam, cyan, indigo, and pale mint palette",
      "rain blue, soft violet, and glassy turquoise palette",
    ],
    Fire: [
      "ember orange, warm gold, charcoal, and coral palette",
      "rose red, brass gold, smoke gray, and cream palette",
      "lantern yellow, burnt sienna, and crimson palette",
    ],
    Earth: [
      "moss green, bark brown, clay, and pale stone palette",
      "fern green, ochre, slate, and ivory palette",
      "sage, mineral gray, root brown, and amber palette",
    ],
  };
  return options[element][seed % 3];
}

function markingsForPassive(passive) {
  switch (passive) {
    case "Finds small loot":
      return "tiny coin-like freckles and bright curious eyes";
    case "Resists poison":
      return "protective leaf-vein markings and hardy shell patches";
    case "Detects eggs":
      return "soft glowing speckles and nest-shaped motifs";
    case "Finds stone":
      return "mineral chips, pebble spots, and sturdy facets";
    case "Burns brambles":
      return "warm ember streaks and singed vine patterns";
    case "Soothes injuries":
      return "gentle healer markings and soft crescent highlights";
    default:
      return "small unique lineage markings";
  }
}

function accessoryForTownSkill(townSkill) {
  switch (townSkill) {
    case "Hatchery helper":
      return "small hatchery charm tied with twine";
    case "Farming":
      return "sprout charm and garden soil details";
    case "Lighting":
      return "tiny lantern glow accents";
    case "Guarding":
      return "simple camp guard neckerchief";
    case "Workshop heat":
      return "little brass workshop tag and warm sparks";
    case "Hatching":
      return "egg-warming sash and soft nest feathers";
    default:
      return "small handmade camp charm";
  }
}

function moodForTemperament(temperament) {
  switch (temperament) {
    case "Loyal":
      return "loyal, alert, and companionable expression";
    case "Patient":
      return "calm, grounded, patient expression";
    case "Curious":
      return "curious, bright, upward-looking expression";
    case "Brave":
      return "brave stance, squared posture, confident eyes";
    case "Restless":
      return "restless energy, playful lean, lively eyes";
    case "Gentle":
      return "gentle healer expression, relaxed posture";
    default:
      return "friendly creature expression";
  }
}
