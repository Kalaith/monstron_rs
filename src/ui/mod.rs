use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub const VIEW_WIDTH: f32 = 1280.0;
pub const VIEW_HEIGHT: f32 = 720.0;
pub const BACKGROUND: Color = Color::new(0.075, 0.086, 0.102, 1.0);
pub const PANEL: Color = Color::new(0.118, 0.137, 0.157, 0.92);
pub const PANEL_EDGE: Color = Color::new(0.275, 0.329, 0.376, 1.0);
pub const TEXT: Color = Color::new(0.827, 0.851, 0.847, 1.0);
pub const TEXT_BRIGHT: Color = Color::new(0.957, 0.965, 0.941, 1.0);
pub const TEXT_DIM: Color = Color::new(0.572, 0.627, 0.627, 1.0);
pub const ACCENT: Color = Color::new(0.604, 0.827, 0.608, 1.0);
pub const WARN: Color = Color::new(0.914, 0.612, 0.369, 1.0);
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
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(color).with_border(2.0, PANEL_EDGE);
    macroquad_toolkit::ui::draw_surface(rect, &surface);

    let text_color = if enabled { TEXT_BRIGHT } else { TEXT_DIM };
    let font_size = if rect.h < 30.0 || rect.w < 80.0 {
        18.0
    } else if rect.h < 38.0 {
        20.0
    } else {
        24.0
    };
    macroquad_toolkit::ui::draw_text_centered_in_box(
        label, rect.x, rect.y, rect.w, rect.h, font_size, text_color,
    );
}

pub fn draw_title_button(rect: Rect, label: &str, enabled: bool) {
    let hovered = enabled && is_mouse_over(rect);
    let fill = if !enabled {
        Color::from_rgba(18, 24, 29, 210)
    } else if hovered {
        Color::from_rgba(30, 59, 55, 235)
    } else {
        Color::from_rgba(16, 31, 38, 225)
    };
    let outer_edge = if hovered {
        Color::from_rgba(190, 232, 166, 255)
    } else if enabled {
        Color::from_rgba(107, 138, 119, 235)
    } else {
        Color::from_rgba(63, 73, 75, 210)
    };
    let warm_edge = if hovered {
        Color::from_rgba(234, 181, 91, 255)
    } else {
        Color::from_rgba(143, 103, 61, 225)
    };

    draw_rectangle(
        rect.x + 5.0,
        rect.y + 6.0,
        rect.w,
        rect.h,
        Color::from_rgba(2, 6, 9, 170),
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, outer_edge);
    draw_line(
        rect.x + 10.0,
        rect.y + 4.0,
        rect.x + rect.w - 10.0,
        rect.y + 4.0,
        1.0,
        Color::from_rgba(214, 244, 191, 70),
    );
    draw_line(
        rect.x + 12.0,
        rect.y + rect.h - 4.0,
        rect.x + rect.w - 12.0,
        rect.y + rect.h - 4.0,
        1.5,
        warm_edge,
    );

    let text_color = if enabled {
        Color::from_rgba(230, 248, 218, 255)
    } else {
        Color::from_rgba(127, 143, 137, 255)
    };
    macroquad_toolkit::ui::draw_text_centered_in_box(
        label, rect.x, rect.y, rect.w, rect.h, 22.0, text_color,
    );
}

pub fn draw_toggle(rect: Rect, label: &str, enabled: bool) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(PANEL).with_border(1.5, PANEL_EDGE);
    macroquad_toolkit::ui::draw_surface(rect, &surface);

    draw_ui_text_ex(
        label,
        rect.x + 20.0,
        rect.y + 36.0,
        TextParams {
            font_size: 24,
            color: TEXT_BRIGHT,
            ..Default::default()
        },
    );

    let track_h = 30.0;
    let track_w = 76.0;
    let track_x = rect.x + rect.w - track_w - 20.0;
    let track_y = rect.y + rect.h * 0.5 - track_h * 0.5;
    let track_color = if enabled { ACCENT } else { BUTTON_DISABLED };
    let knob_x = if enabled {
        track_x + track_w - track_h * 0.5
    } else {
        track_x + track_h * 0.5
    };

    draw_rectangle(
        track_x + track_h * 0.5,
        track_y,
        track_w - track_h,
        track_h,
        track_color,
    );
    draw_circle(
        track_x + track_h * 0.5,
        track_y + track_h * 0.5,
        track_h * 0.5,
        track_color,
    );
    draw_circle(
        track_x + track_w - track_h * 0.5,
        track_y + track_h * 0.5,
        track_h * 0.5,
        track_color,
    );
    draw_circle(knob_x, track_y + track_h * 0.5, track_h * 0.38, TEXT_BRIGHT);
}

pub fn draw_panel(rect: Rect) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(PANEL).with_border(1.5, PANEL_EDGE);
    macroquad_toolkit::ui::draw_surface(rect, &surface);
}

pub fn draw_section_title(title: &str, x: f32, y: f32) {
    draw_ui_text_ex(
        title,
        x,
        y,
        TextParams {
            font_size: 28,
            color: TEXT_BRIGHT,
            ..Default::default()
        },
    );
}

pub fn draw_centered_text(text: &str, center_x: f32, y: f32, font_size: u16, color: Color) {
    macroquad_toolkit::ui::draw_text_centered(
        text,
        center_x,
        y,
        macroquad_toolkit::ui::TextStyle::new(font_size as f32, color),
    );
}

pub fn draw_status(status_message: &str) {
    let rect = Rect::new(24.0, VIEW_HEIGHT - 48.0, VIEW_WIDTH - 48.0, 28.0);
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.063, 0.071, 0.082, 0.86));
    macroquad_toolkit::ui::draw_surface(rect, &surface);
    draw_ui_text_ex(
        status_message,
        rect.x + 12.0,
        rect.y + 20.0,
        TextParams {
            font_size: 20,
            color: TEXT_DIM,
            ..Default::default()
        },
    );
}

fn is_mouse_over(rect: Rect) -> bool {
    let mouse = macroquad_toolkit::ui::virtual_mouse_position(VIEW_WIDTH, VIEW_HEIGHT);
    macroquad_toolkit::input::rect_contains_point(rect, mouse)
}
