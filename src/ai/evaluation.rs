use crate::ai::{get_player_back_rank, get_player_pawn_bits};
use chess::{Board, ChessMove, MoveGen, Piece, ALL_PIECES, NUM_PIECES};
use rayon::prelude::ParallelSliceMut;

const PIECE_VALUE: [u32; NUM_PIECES] = [100, 300, 300, 500, 900, 99900];

pub(super) fn eval_board(board: &Board) -> isize {
    let mut total = 0;
    let max = board.color_combined(board.side_to_move());
    let min = board.color_combined(!board.side_to_move());

    for piece in ALL_PIECES {
        let value = piece_value(piece) as u32;
        let bits = board.pieces(piece);
        total += ((max & bits).popcnt() * value) as isize;
        total -= ((min & bits).popcnt() * value) as isize;
    }

    total
}

pub(super) fn sorted_moves(board: &Board) -> Vec<ChessMove> {
    let gen = MoveGen::new_legal(board);
    let mut moves = gen.collect::<Vec<_>>();
    moves.par_sort_unstable_by_key(|mov| -eval_move(board, *mov));
    moves
}

pub(super) fn eval_move(board: &Board, cmove: ChessMove) -> isize {
    let mut value = 0;
    let moving_piece = board.piece_on(cmove.get_source()).unwrap();
    let captured_piece = board.piece_on(cmove.get_dest());

    // Promoting is often good
    if let Some(promoted) = cmove.get_promotion() {
        value += piece_value(promoted);
    }

    // Capture highest-value opponent pieces with lowest-value pieces first
    if let Some(captured_piece) = captured_piece {
        value += (10 * piece_value(captured_piece)) - piece_value(moving_piece);
    }

    let undeveloped_pawns_count =
        (board.color_combined(board.side_to_move()) & board.pieces(Piece::Pawn) & get_player_pawn_bits(board)).popcnt();
    let is_early_game = undeveloped_pawns_count >= 6;
    // Prioritize developing pawns earlygame
    if is_early_game && moving_piece == Piece::Pawn {
        let file = cmove.get_source().get_file().to_index();
        // Middle pawns first
        value += (8 - isize::abs(file as isize - 4)) * 20;
    }

    // Penalize moving a piece to the back rank to prevent 'undeveloping' pieces during earlygame
    if is_early_game && cmove.get_dest().get_rank() == get_player_back_rank(board) {
        value -= 50;
    }

    // Penalize developing the queen too early
    if undeveloped_pawns_count > 6 && moving_piece == Piece::Queen {
        value -= 25;
    }

    // Introduce some random variation to prevent repetition (AI chooses first move if multiple 'ideal' moves found)
    let rand = rand::random::<f32>();
    let rand = rand * 10.0;
    value + rand as isize
}

fn piece_value(piece: Piece) -> isize {
    PIECE_VALUE[piece.to_index()] as isize
}

#[cfg(test)]
mod tests {
    use super::eval_board;
    use chess::Board;

    #[test]
    fn new_board_is_0() {
        let board = Board::default();
        assert_eq!(eval_board(&board), 0);
    }
}
