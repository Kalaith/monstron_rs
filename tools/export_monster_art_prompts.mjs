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
      "blank image, empty image, whiteout, overexposed, low contrast, photorealistic, 3d render, toy figurine, sculpture, plastic material, human, humanoid, anime girl, multiple creatures, detailed scenery, busy background, UI, HUD, text, logo, watermark, cropped, blurry, noisy, extra limbs, harsh shadow, gore",
  };
}

function buildPrompt(profile, lineage, quality) {
  return [
    "single friendly tower-born monster, 2D game creature asset",
    "centered full body, compact readable silhouette, visible creature, high contrast, three-quarter view",
    "plain warm gray studio background, no scenery, no props except the creature charm",
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
