use crate::graphics::System;
use chess::Game;

mod graphics;
pub mod minimax;

fn main() {
    let game = Game::new();
    System::start(game);
}
