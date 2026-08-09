#![allow(clippy::too_many_arguments)]

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod assets;
mod data;
mod engine;
mod facility_actions;
mod game;
mod save;
mod screens;
mod state;
mod ui;

use game::Game;

fn window_conf() -> Conf {
    // Hand-built Conf means no automatic arming: without this the capture run
    // puts a full game window on the desktop for its whole duration.
    capture::headless::arm("MONSTRON");

    // Built by hand (not capture::capture_window_conf) to keep sample_count: 0
    // and high_dpi: false, which the game already relies on; still honors
    // MONSTRON_WINDOW_WIDTH/HEIGHT overrides for the capture harness.
    Conf {
        window_title: "Hatchspire".to_owned(),
        window_width: capture::env_i32("MONSTRON_WINDOW_WIDTH", 1280),
        window_height: capture::env_i32("MONSTRON_WINDOW_HEIGHT", 720),
        window_resizable: true,
        high_dpi: false,
        sample_count: 0,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: when MONSTRON_CAPTURE_PATH is set, seed a scene,
    // simulate deterministic frames, write a PNG, and exit.
    if let Some(configs) = capture::CaptureConfig::all_from_env("MONSTRON") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |_dt| {
                game.update();
                game.draw();
            })
            .await;
        }
        return;
    }

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
