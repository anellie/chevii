use crate::graphics::Graphics;
use crate::uci_engine::UCIEngine;
use chess::{ChessMove, Game, MoveGen, Piece, Square, ALL_SQUARES};
use std::env;

pub mod ai;
mod graphics;
mod uci_engine;

fn main() {
    let other_engine = env::args()
        .find(|s| s == "--stockfish")
        .map(|_| UCIEngine::new_stockfish());
    let game = Game::new();
    System::start(game, other_engine);
}

pub struct System {
    pub game: Game,
    pub gui: Graphics,
    pub info: GameInfo,
    pub other_engine: Option<UCIEngine>,
}

impl System {
    fn square_clicked(&mut self, square: usize) {
        let board = self.game.current_position();

        match self.info.selected_square {
            Some(prev_square) if prev_square == square => {
                self.info.selected_square = None;
                return;
            }

            Some(prev_square) => {
                let move_ = ChessMove::new(sq(prev_square), sq(square), None);
                let move_promote = ChessMove::new(sq(prev_square), sq(square), Some(Piece::Queen));

                let mut make_move = |m: ChessMove| {
                    if board.legal(m) {
                        self.game.make_move(m);
                        self.make_ai_move();
                        self.info.selected_square = None;
                        self.info.moves_count += 1;
                    }
                };

                make_move(move_);
                make_move(move_promote);
            }

            _ => (),
        }

        if board.color_on(sq(square)) == Some(board.side_to_move()) {
            self.info.selected_square = Some(square);
        }
    }

    fn possible_moves(&mut self) -> MoveGen {
        MoveGen::new_legal(&self.game.current_position())
    }

    fn make_ai_move(&mut self) {
        let to_make = if self.info.moves_count == 0 {
            // E5 as opening
            ChessMove::new(sq(52), sq(36), None)
        } else {
            ai::get_best_move(&self.game.current_position())
        };
        self.game.make_move(to_make);
        self.info.last_move = Some(to_make);
        self.info.moves_count += 1;
    }
}

#[derive(Default)]
pub struct GameInfo {
    pub last_move: Option<ChessMove>,
    pub moves_count: usize,
    pub selected_square: Option<usize>,
}

fn sq(idx: usize) -> Square {
    ALL_SQUARES[idx]
}
