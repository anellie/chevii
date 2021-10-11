#![feature(test)]

extern crate test;

use crate::graphics::Graphics;
use crate::uci_engine::UCIEngine;
use chess::{ChessMove, Game, MoveGen, Piece, Square, ALL_SQUARES, BoardStatus};
use structopt::StructOpt;

pub mod ai;
mod graphics;
mod uci_engine;

#[derive(StructOpt, Debug)]
struct Opt {
    /// Have the AI play against an UCI engine
    #[structopt(short, long)]
    engine_path: Option<String>,

    /// Run a single AI move calculation and exit
    #[structopt(short, long)]
    bench: bool,
}

fn main() {
    env_logger::init();
    let opts = Opt::from_args();

    if opts.bench {
        bench();
    } else {
        let other_engine = opts.engine_path.map(|eng| UCIEngine::new(&eng));
        System::start(Game::new(), other_engine);
    }
}

fn bench() {
    let mut game = Game::new();
    game.make_move(ChessMove::new(sq(12), sq(28), None)); // e2e4
    game.make_move(ChessMove::new(sq(52), sq(36), None)); // e7e5
    game.make_move(ChessMove::new(sq(6), sq(21), None)); // Ng1f5
    let board = game.current_position();
    ai::get_best_move(&board);
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
        let pos = self.game.current_position();
        if pos.status() == BoardStatus::Checkmate {
            return;
        }

        let to_make = if self.info.moves_count == 0 {
            // E5 as opening
            ChessMove::new(sq(52), sq(36), None)
        } else {
            ai::get_best_move(&pos)
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
