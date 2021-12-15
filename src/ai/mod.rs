mod evaluation;
mod minimax;
pub mod book;
mod table;

use chess::{BitBoard, Color, Rank, Board, ChessMove};

type RatedMove = (ChessMove, isize);

pub fn get_best_move(board: Board, time: f32) -> ChessMove {
    if let Some(mov) = book::get_for(&board) {
        mov
    } else {
        minimax::calculate_move(board, time)
    }
}

fn get_player_back_rank(board: &Board) -> Rank {
    match board.side_to_move() {
        Color::White => Rank::First,
        Color::Black => Rank::Eighth
    }
}

fn get_player_pawn_bits(board: &Board) -> BitBoard {
    match board.side_to_move() {
        Color::White => BitBoard(0b0000000000000000000000000000000000000000000000001111111100000000),
        Color::Black => BitBoard(0b0000000011111111000000000000000000000000000000000000000000000000)
    }
}
