use crate::graphics::System;
use chess::Game;

mod graphics;

fn main() {
    let game = Game::new();
    System::start(game);
}
