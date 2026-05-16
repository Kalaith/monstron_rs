use macroquad::prelude::*;

pub fn draw_monster_badge(seed: u64, x: f32, y: f32, size: f32) {
    let (body, shine) = colors_from_seed(seed);
    draw_circle(x + size * 0.5, y + size * 0.58, size * 0.42, body);
    draw_circle(x + size * 0.36, y + size * 0.38, size * 0.14, shine);
    draw_circle(x + size * 0.37, y + size * 0.54, size * 0.045, BLACK);
    draw_circle(x + size * 0.62, y + size * 0.54, size * 0.045, BLACK);
}

pub fn draw_egg_badge(seed: u64, x: f32, y: f32, size: f32) {
    let (shell, spots) = colors_from_seed(seed ^ 0xE66);
    draw_circle(x + size * 0.5, y + size * 0.58, size * 0.38, shell);
    draw_circle(x + size * 0.5, y + size * 0.42, size * 0.28, shell);
    draw_circle(x + size * 0.38, y + size * 0.52, size * 0.06, spots);
    draw_circle(x + size * 0.58, y + size * 0.62, size * 0.05, spots);
    draw_circle(x + size * 0.52, y + size * 0.42, size * 0.04, spots);
}

fn colors_from_seed(seed: u64) -> (Color, Color) {
    let r = 80 + ((seed >> 8) & 95) as u8;
    let g = 130 + ((seed >> 16) & 85) as u8;
    let b = 150 + ((seed >> 24) & 75) as u8;
    let shine = Color::from_rgba(
        r.saturating_add(70),
        g.saturating_add(45),
        b.saturating_add(35),
        255,
    );
    (Color::from_rgba(r, g, b, 255), shine)
}
