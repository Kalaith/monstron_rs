use macroquad::prelude::*;

pub const VIEW_WIDTH: f32 = 1280.0;
pub const VIEW_HEIGHT: f32 = 720.0;
pub const BACKGROUND: Color = Color::new(0.075, 0.086, 0.102, 1.0);
pub const PANEL: Color = Color::new(0.118, 0.137, 0.157, 0.92);
pub const PANEL_EDGE: Color = Color::new(0.275, 0.329, 0.376, 1.0);
pub const TEXT: Color = Color::new(0.827, 0.851, 0.847, 1.0);
pub const TEXT_BRIGHT: Color = Color::new(0.957, 0.965, 0.941, 1.0);
pub const TEXT_DIM: Color = Color::new(0.572, 0.627, 0.627, 1.0);
pub const ACCENT: Color = Color::new(0.604, 0.827, 0.608, 1.0);
const BUTTON: Color = Color::new(0.173, 0.243, 0.275, 1.0);
const BUTTON_HOVER: Color = Color::new(0.224, 0.337, 0.365, 1.0);
const BUTTON_DISABLED: Color = Color::new(0.145, 0.157, 0.169, 1.0);

pub fn button_clicked(rect: Rect, enabled: bool) -> bool {
    enabled && is_mouse_over(rect) && is_mouse_button_released(MouseButton::Left)
}

pub fn virtual_camera() -> Camera2D {
    Camera2D {
        target: vec2(VIEW_WIDTH * 0.5, VIEW_HEIGHT * 0.5),
        zoom: vec2(2.0 / VIEW_WIDTH, 2.0 / VIEW_HEIGHT),
        ..Default::default()
    }
}

pub fn draw_button(rect: Rect, label: &str, enabled: bool) {
    let hovered = enabled && is_mouse_over(rect);
    let color = if !enabled {
        BUTTON_DISABLED
    } else if hovered {
        BUTTON_HOVER
    } else {
        BUTTON
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, PANEL_EDGE);

    let text_color = if enabled { TEXT_BRIGHT } else { TEXT_DIM };
    let font_size = if rect.h < 30.0 || rect.w < 96.0 {
        16
    } else if rect.h < 38.0 {
        18
    } else {
        22
    };
    let measured = measure_text(label, None, font_size, 1.0);
    draw_text_ex(
        label,
        rect.x + rect.w * 0.5 - measured.width * 0.5,
        rect.y + rect.h * 0.5 + measured.height * 0.38,
        TextParams {
            font_size,
            color: text_color,
            ..Default::default()
        },
    );
}

pub fn draw_panel(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, PANEL_EDGE);
}

pub fn draw_section_title(title: &str, x: f32, y: f32) {
    draw_text_ex(
        title,
        x,
        y,
        TextParams {
            font_size: 25,
            color: TEXT_BRIGHT,
            ..Default::default()
        },
    );
}

pub fn draw_centered_text(text: &str, center_x: f32, y: f32, font_size: u16, color: Color) {
    let measured = measure_text(text, None, font_size, 1.0);
    draw_text_ex(
        text,
        center_x - measured.width * 0.5,
        y,
        TextParams {
            font_size,
            color,
            ..Default::default()
        },
    );
}

pub fn draw_status(status_message: &str) {
    let rect = Rect::new(24.0, VIEW_HEIGHT - 48.0, VIEW_WIDTH - 48.0, 28.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.063, 0.071, 0.082, 0.86),
    );
    draw_text_ex(
        status_message,
        rect.x + 12.0,
        rect.y + 20.0,
        TextParams {
            font_size: 18,
            color: TEXT_DIM,
            ..Default::default()
        },
    );
}

fn is_mouse_over(rect: Rect) -> bool {
    let (x, y) = virtual_mouse_position();
    x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h
}

fn virtual_mouse_position() -> (f32, f32) {
    let (x, y) = mouse_position();
    let scale_x = VIEW_WIDTH / screen_width().max(1.0);
    let scale_y = VIEW_HEIGHT / screen_height().max(1.0);
    (x * scale_x, y * scale_y)
}
