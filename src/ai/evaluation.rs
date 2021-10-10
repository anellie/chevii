use crate::ai::{OPPONENT, PLAYER, PLAYER_PAWN_RANK_BITS, PLAYER_BACK_RANK};
use chess::{Board, ALL_PIECES, NUM_PIECES, ChessMove, Piece, MoveGen, BitBoard};

const PIECE_VALUE: [u32; NUM_PIECES] = [100, 300, 300, 500, 900, 99900];

pub(super) fn eval_board(board: &Board) -> isize {
    let mut total = 0;
    let max = board.color_combined(PLAYER);
    let min = board.color_combined(OPPONENT);

    for piece in ALL_PIECES {
        let value = piece_value(piece) as u32;
        let bits = board.pieces(piece);
        total += ((max & bits).popcnt() * value) as isize;
        total -= ((min & bits).popcnt() * value) as isize;
    }

    total
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

    let undeveloped_pawns_count = (board.color_combined(PLAYER) & board.pieces(Piece::Pawn) & PLAYER_PAWN_RANK_BITS).popcnt();
    let is_early_game = undeveloped_pawns_count >= 6;
    // Prioritize developing pawns earlygame
    if is_early_game && moving_piece == Piece::Pawn {
        value += 50;
    }

    // Penalize moving a piece to the back rank to prevent 'undeveloping' pieces during earlygame
    if is_early_game && cmove.get_dest().get_rank() == PLAYER_BACK_RANK {
        value -= 50;
    }

    // Penalize developing the queen too early
    if undeveloped_pawns_count > 6 && moving_piece == Piece::Queen {
        value -= 25;
    }

    // Penalize exposing the moving piece to an enemy attack
    let post_board = board.make_move_new(cmove);
    let mut oppo_moves = MoveGen::new_legal(&post_board);
    oppo_moves.set_iterator_mask(BitBoard::from_square(cmove.get_dest()));
    value -= piece_value(moving_piece) * oppo_moves.count() as isize;

    // Introduce some random variation to prevent repetition (AI chooses first move if multiple 'ideal' moves found)
    //let rand = rand::random::<f32>();
    //let rand = rand * 10.0;
    value //+ rand as isize
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
