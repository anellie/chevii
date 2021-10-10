mod evaluation;
mod minimax;

use chess::Color;
pub use minimax::get_best_move;

const PLAYER: Color = Color::Black;
const OPPONENT: Color = Color::White;
