use std::cell::RefCell;
use std::collections::HashMap;

use macroquad::prelude::*;

const MONSTERS: &str = "monsters";
const EGGS: &str = "eggs";
const ENEMIES: &str = "enemies";
const LANDMARKS: &str = "landmarks";
const NPCS: &str = "npcs";
const ROOMS: &str = "rooms";
const VFX: &str = "vfx";
const DUNGEON_FEATURES: &str = "dungeon_features";
const DUNGEON_ENEMIES: &str = "dungeon_enemies";
const PARTY_MARKERS: &str = "party_markers";

thread_local! {
    static TEXTURES: RefCell<HashMap<&'static str, Texture2D>> = RefCell::new(HashMap::new());
}

/// Draws a canonical companion portrait. The six slots are deliberately stable:
/// Slime, Rootling, Glowmoth, Pebblepup, Emberkit, and Rillfin.
pub fn draw_monster_badge(species_id: &str, x: f32, y: f32, size: f32) {
    let index = match species_id {
        "slime" => 0,
        "rillfin" => 1,
        "rootling" => 2,
        "emberkit" => 3,
        "pebblepup" => 4,
        "glowmoth" => 5,
        _ => 0,
    };
    draw_atlas(MONSTERS, 2, 3, index, x, y, size, size);
}

/// Draws one of the twelve named egg designs, rather than a seed-coloured blob.
pub fn draw_egg_badge(egg_type_id: &str, x: f32, y: f32, size: f32) {
    let index = match egg_type_id {
        "mossy_egg" => 0,
        "glimmer_egg" => 1,
        "pebble_egg" => 2,
        "ember_egg" => 3,
        "ripple_egg" => 4,
        "rootbound_egg" => 5,
        "lantern_egg" => 6,
        "garden_egg" => 7,
        "ore_veined_egg" => 8,
        "boss_egg" => 9,
        "moonlit_egg" => 10,
        "sunken_egg" => 11,
        _ => 0,
    };
    draw_atlas(EGGS, 4, 3, index, x, y, size, size);
}

pub fn draw_enemy_badge(enemy_id: &str, x: f32, y: f32, size: f32) {
    let index = match enemy_id {
        "moss_mite" => 0,
        "lamp_gnat" => 1,
        "root_snapper" => 2,
        "ember_wisp" => 3,
        "glass_leech" => 4,
        "hushed_sentry" => 5,
        "garden_shade" => 0,
        "iron_rook" => 5,
        "verdant_crown" => 2,
        _ => 0,
    };
    draw_atlas(ENEMIES, 2, 3, index, x, y, size, size);
}

pub fn draw_landmark(index: usize, x: f32, y: f32, width: f32, height: f32) {
    draw_atlas(LANDMARKS, 2, 3, index % 6, x, y, width, height);
}

pub fn draw_npc(npc_id: &str, x: f32, y: f32, size: f32) {
    let index = match npc_id {
        "mara" => 0,
        "bram" => 1,
        "lio" => 2,
        _ => 0,
    };
    draw_atlas(NPCS, 3, 2, index, x, y, size, size);
}

pub fn draw_room_vignette(floor: u32, x: f32, y: f32, width: f32, height: f32) {
    draw_atlas(
        ROOMS,
        2,
        3,
        (floor.saturating_sub(1) as usize / 2) % 6,
        x,
        y,
        width,
        height,
    );
}

pub fn draw_combat_vfx(index: usize, x: f32, y: f32, size: f32) {
    draw_atlas(VFX, 3, 2, index % 6, x, y, size, size);
}

/// Draws the tower's shared landmark vocabulary: nest, cache, stairs, crystal
/// threshold, recovery spring, and ancient machinery.
pub fn draw_dungeon_feature(index: usize, x: f32, y: f32, width: f32, height: f32) {
    draw_atlas(DUNGEON_FEATURES, 2, 3, index % 6, x, y, width, height);
}

pub fn draw_dungeon_enemy(index: usize, x: f32, y: f32, width: f32, height: f32) {
    draw_atlas(DUNGEON_ENEMIES, 3, 2, index % 6, x, y, width, height);
}

pub fn draw_party_marker(index: usize, x: f32, y: f32, width: f32, height: f32) {
    draw_atlas(PARTY_MARKERS, 3, 2, index % 6, x, y, width, height);
}

fn draw_atlas(
    asset: &'static str,
    columns: usize,
    rows: usize,
    index: usize,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let texture = texture(asset);
    let cell_w = texture.width() / columns as f32;
    let cell_h = texture.height() / rows as f32;
    let column = index % columns;
    let row = index / columns;
    draw_texture_ex(
        &texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            source: Some(Rect::new(
                column as f32 * cell_w,
                row as f32 * cell_h,
                cell_w,
                cell_h,
            )),
            ..Default::default()
        },
    );
}

fn texture(asset: &'static str) -> Texture2D {
    TEXTURES.with(|textures| {
        let mut textures = textures.borrow_mut();
        textures
            .entry(asset)
            .or_insert_with(|| Texture2D::from_file_with_format(asset_bytes(asset), None))
            .clone()
    })
}

fn asset_bytes(asset: &str) -> &'static [u8] {
    match asset {
        MONSTERS => {
            include_bytes!("../../assets/generated/monster_art/monster_sprite_atlas_v1.png")
        }
        EGGS => include_bytes!("../../assets/generated/monster_art/canonical_egg_atlas_v1.png"),
        ENEMIES => {
            include_bytes!("../../assets/generated/monster_art/enemy_boss_sprite_v2_atlas.png")
        }
        LANDMARKS => {
            include_bytes!("../../assets/generated/town/town_facility_landmarks_v4_atlas.png")
        }
        NPCS => include_bytes!("../../assets/generated/town/town_service_npcs_v5_atlas.png"),
        ROOMS => include_bytes!(
            "../../assets/generated/dungeon/dungeon_deep_room_vignettes_v5_atlas.png"
        ),
        VFX => include_bytes!("../../assets/generated/combat/combat_vfx_atlas_chroma_v1.png"),
        DUNGEON_FEATURES => {
            include_bytes!("../../assets/generated/dungeon/dungeon_sprite_atlas_v1.png")
        }
        DUNGEON_ENEMIES => {
            include_bytes!("../../assets/generated/dungeon/dungeon_enemy_atlas_v1.png")
        }
        PARTY_MARKERS => include_bytes!(
            "../../assets/generated/monster_art/monster_party_marker_travel_v5_atlas.png"
        ),
        _ => unreachable!("unknown visual asset"),
    }
}
