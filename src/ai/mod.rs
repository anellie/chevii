mod evaluation;
mod minimax;
mod nnue;
mod statistics;
mod table;

use crate::ai::statistics::Stat;
use chess::{BitBoard, Board, ChessMove, Color, MoveGen, Rank};
use rayon::slice::ParallelSliceMut;

type RatedMove = (ChessMove, isize);

pub fn get_best_move(board: Board, time: f32) -> ChessMove {
    nnue::init();
    let mov = minimax::calculate_move(board, time);
    Stat::log();
    mov
}

fn sorted_moves(board: &Board) -> Vec<RatedMove> {
    let gen = MoveGen::new_legal(board);
    let mut moves = gen
        .map(|m| (m, evaluation::eval_move(board, m)))
        .collect::<Vec<_>>();
    moves.par_sort_unstable_by_key(|mov| -mov.1);
    moves
}

fn capturing_moves(board: &Board) -> Vec<RatedMove> {
    let mut gen = MoveGen::new_legal(board);
    gen.set_iterator_mask(*board.color_combined(!board.side_to_move()));
    let mut moves = gen
        .map(|m| (m, evaluation::eval_move(board, m)))
        .collect::<Vec<_>>();
    moves.par_sort_unstable_by_key(|mov| -mov.1);
    moves
}

fn get_player_back_rank(board: &Board) -> Rank {
    match board.side_to_move() {
        Color::White => Rank::First,
        Color::Black => Rank::Eighth,
    }
}

fn get_player_pawn_bits(board: &Board) -> BitBoard {
    match board.side_to_move() {
        Color::White => {
            BitBoard(0b0000000000000000000000000000000000000000000000001111111100000000)
        }
        Color::Black => {
            BitBoard(0b0000000011111111000000000000000000000000000000000000000000000000)
        }
    }
}
