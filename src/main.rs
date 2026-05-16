use macroquad::prelude::*;

mod assets;
mod data;
mod engine;
mod game;
mod save;
mod screens;
mod state;
mod ui;

use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Hatchspire".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: false,
        sample_count: 0,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
