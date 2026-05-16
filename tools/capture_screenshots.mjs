import { spawn } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const chromePath = "C:/Program Files/Google/Chrome/Application/chrome.exe";
const remotePort = 61234;
const previewUrl = process.argv[2] ?? "http://127.0.0.1:58853/";
const outDir = resolve("screenshots", "phase8-hardening");

const keyCodes = {
  b: ["b", "KeyB", 66],
  d: ["d", "KeyD", 68],
  h: ["h", "KeyH", 72],
  l: ["l", "KeyL", 76],
  r: ["r", "KeyR", 82],
  t: ["t", "KeyT", 84],
  w: ["w", "KeyW", 87],
  Enter: ["Enter", "Enter", 13],
  Escape: ["Escape", "Escape", 27],
  Space: [" ", "Space", 32],
};

const preparedSave = {
  slot: { name: "slot_1", save_date: "Unknown", version: "0.1.0" },
  data: {
    version: 1,
    state: {
      day: 1,
      resources: {
        stacks: [
          { resource_id: "coins", amount: 80 },
          { resource_id: "wood", amount: 60 },
          { resource_id: "stone", amount: 50 },
          { resource_id: "ore", amount: 8 },
          { resource_id: "herbs", amount: 30 },
          { resource_id: "crystal", amount: 4 },
        ],
      },
      town: {
        buildings: [
          { building_id: "house", level: 2 },
          { building_id: "hatchery", level: 1 },
          { building_id: "stable", level: 1 },
          { building_id: "breeding_grove", level: 1 },
          { building_id: "workshop", level: 1 },
          { building_id: "shop", level: 1 },
        ],
        assignments: [
          { monster_id: 1, job: "Forage" },
          { monster_id: 2, job: "HatcheryCare" },
        ],
      },
      monster_roster: {
        next_id: 4,
        monsters: [
          monster(1, "Pip", "slime", "Water", "Loyal", "Scout", "Finds small loot", "Hatchery helper", 3, 7, 0x51a15eed),
          monster(2, "Ripple", "rillfin", "Water", "Gentle", "Support", "Soothes injuries", "Hatching", 2, 5, 0xbee57001),
          monster(3, "Ember", "emberkit", "Fire", "Restless", "Striker", "Burns brambles", "Workshop heat", 2, 4, 0xf17e2002),
        ],
        party_slots: [1, 2, 3, null, null, null],
      },
      egg_inventory: {
        next_id: 2,
        eggs: [
          {
            id: 1,
            egg_type_id: "mossy_egg",
            days_remaining: 1,
            origin_floor: 3,
            palette_seed: 0xa7c40001,
            inheritance: {
              parent_ids: [1, 2],
              species_options: ["slime", "rillfin"],
              element_options: ["Water"],
              temperament_options: ["Loyal", "Gentle"],
              passive_options: ["Finds small loot", "Soothes injuries"],
              mutation_floor: 3,
              mutated: false,
            },
          },
        ],
      },
      tower_progress: { best_floor: 1, unlocked_floor: 1 },
      tower_run: null,
      combat: null,
      npc_relationships: [
        { npc_id: "mara", friendship: 2 },
        { npc_id: "bram", friendship: 1 },
        { npc_id: "lio", friendship: 1 },
      ],
      story_flags: {
        flags: ["arrived_at_tower_camp", "starter_slime_breeding_bond"],
      },
      activity_log: {
        next_id: 4,
        entries: [
          { id: 1, day: 1, message: "A ruined tower rises above the camp." },
          { id: 2, day: 1, message: "Pip the slime waits beside a cold hatchery brazier." },
          { id: 3, day: 1, message: "Workshop jobs are ready for the morning." },
        ],
      },
    },
  },
};

const shots = [
  ["01_main_menu", []],
  ["02_town", ["l"]],
  ["03_hatchery", ["l", "h"]],
  ["04_stable", ["l", "r"]],
  ["05_breeding_grove", ["l", "b"]],
  ["06_workshop", ["l", "w"]],
  ["07_shop", ["l", "t"]],
  ["08_dungeon_prep", ["l", "d"]],
  ["09_tower", ["l", "d", "Enter"]],
  ["10_combat", ["l", "d", "Enter", "Space"]],
  ["11_end_of_day", ["l", "Space"]],
  ["12_town_menu", ["l", "Escape"]],
];

async function main() {
  await rm(outDir, { recursive: true, force: true });
  await mkdir(outDir, { recursive: true });
  const userDataDir = await mkdtemp(join(tmpdir(), "monstron-chrome-"));

  const chrome = spawn(chromePath, [
    "--headless=new",
    "--disable-background-networking",
    "--disable-extensions",
    "--disable-sync",
    "--mute-audio",
    "--no-first-run",
    "--remote-debugging-address=127.0.0.1",
    `--remote-debugging-port=${remotePort}`,
    `--user-data-dir=${userDataDir}`,
    "--window-size=1320,900",
    "about:blank",
  ], { stdio: "ignore" });

  try {
    const endpoint = await waitForDebugEndpoint();
    const cdp = await CdpClient.connect(endpoint.webSocketDebuggerUrl);
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("DOM.enable");

    await cdp.send("Page.addScriptToEvaluateOnNewDocument", {
      source: `localStorage.setItem("save_slot_1", ${JSON.stringify(JSON.stringify(preparedSave))});`,
    });

    for (const [name, keys] of shots) {
      await loadPreview(cdp);
      await wait(1200);
      await focusCanvas(cdp);
      for (const key of keys) {
        await pressKey(cdp, key);
        await wait(450);
      }
      await wait(900);
      await captureCanvas(cdp, join(outDir, `${name}.png`));
    }

    console.log(`Saved ${shots.length} screenshots to ${outDir}`);
  } finally {
    chrome.kill();
    await wait(500);
    await rm(userDataDir, { recursive: true, force: true });
  }
}

function monster(
  id,
  name,
  species_id,
  element,
  temperament,
  role,
  passive,
  town_skill,
  level,
  bond,
  visual_seed,
) {
  const speciesStats = {
    slime: [18, 4, 3, 6],
    rillfin: [20, 4, 4, 6],
    emberkit: [19, 8, 3, 7],
  };
  const [max_hp, attack, defense, speed] = speciesStats[species_id];
  return {
    id,
    name,
    species_id,
    element,
    temperament,
    role,
    passive,
    town_skill,
    level,
    xp: 0,
    bond,
    max_hp,
    hp: max_hp,
    attack,
    defense,
    speed,
    visual_seed,
  };
}

async function loadPreview(cdp) {
  const loaded = cdp.waitFor("Page.loadEventFired");
  await cdp.send("Page.navigate", { url: previewUrl });
  await loaded;
}

async function focusCanvas(cdp) {
  await cdp.send("Runtime.evaluate", {
    expression: `document.getElementById("glcanvas").focus(); true;`,
  });
}

async function pressKey(cdp, keyName) {
  const [key, code, windowsVirtualKeyCode] = keyCodes[keyName];
  const base = {
    key,
    code,
    windowsVirtualKeyCode,
    nativeVirtualKeyCode: windowsVirtualKeyCode,
  };
  await cdp.send("Input.dispatchKeyEvent", { ...base, type: "keyDown" });
  await cdp.send("Input.dispatchKeyEvent", { ...base, type: "keyUp" });
}

async function captureCanvas(cdp, path) {
  const rectResult = await cdp.send("Runtime.evaluate", {
    returnByValue: true,
    expression: `(() => {
      const canvas = document.getElementById("glcanvas");
      const rect = canvas.getBoundingClientRect();
      return { x: rect.x, y: rect.y, width: rect.width, height: rect.height, scale: 1 };
    })()`,
  });
  const shot = await cdp.send("Page.captureScreenshot", {
    format: "png",
    fromSurface: true,
    captureBeyondViewport: false,
    clip: rectResult.result.value,
  });
  await writeFile(path, Buffer.from(shot.data, "base64"));
}

async function waitForDebugEndpoint() {
  const url = `http://127.0.0.1:${remotePort}/json/new?about:blank`;
  for (let attempt = 0; attempt < 80; attempt += 1) {
    try {
      const response = await fetch(url, { method: "PUT" });
      if (response.ok) {
        return await response.json();
      }
    } catch {
      // Chrome is still starting.
    }
    await wait(100);
  }
  throw new Error("Chrome remote debugging endpoint did not start.");
}

function wait(ms) {
  return new Promise((resolveWait) => setTimeout(resolveWait, ms));
}

class CdpClient {
  static async connect(url) {
    const socket = new WebSocket(url);
    await new Promise((resolveOpen, rejectOpen) => {
      socket.addEventListener("open", resolveOpen, { once: true });
      socket.addEventListener("error", rejectOpen, { once: true });
    });
    return new CdpClient(socket);
  }

  constructor(socket) {
    this.socket = socket;
    this.nextId = 1;
    this.pending = new Map();
    this.waiters = new Map();
    socket.addEventListener("message", (event) => this.handleMessage(event));
  }

  send(method, params = {}) {
    const id = this.nextId;
    this.nextId += 1;
    this.socket.send(JSON.stringify({ id, method, params }));
    return new Promise((resolveSend, rejectSend) => {
      this.pending.set(id, { resolve: resolveSend, reject: rejectSend, method });
    });
  }

  waitFor(method) {
    return new Promise((resolveWait) => {
      const waiters = this.waiters.get(method) ?? [];
      waiters.push(resolveWait);
      this.waiters.set(method, waiters);
    });
  }

  handleMessage(event) {
    const message = JSON.parse(event.data);
    if (message.id) {
      const pending = this.pending.get(message.id);
      if (!pending) return;
      this.pending.delete(message.id);
      if (message.error) {
        pending.reject(new Error(`${pending.method}: ${message.error.message}`));
      } else {
        pending.resolve(message.result ?? {});
      }
      return;
    }

    const waiters = this.waiters.get(message.method);
    if (waiters?.length) {
      const resolveWait = waiters.shift();
      resolveWait(message.params ?? {});
    }
  }
}

await main();
