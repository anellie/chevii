use crate::ai::{OPPONENT, PLAYER};
use chess::{Board, ALL_PIECES, NUM_PIECES};

const PIECE_VALUE: [u32; NUM_PIECES] = [100, 300, 300, 500, 900, 99900];

pub(super) fn eval_board(board: &Board) -> isize {
    let mut total = 0;
    let max = board.color_combined(PLAYER);
    let min = board.color_combined(OPPONENT);

    for piece in ALL_PIECES {
        let value = PIECE_VALUE[piece.to_index()];
        let bits = board.pieces(piece);
        total += ((max ^ bits).popcnt() * value) as isize;
        total -= ((min ^ bits).popcnt() * value) as isize;
    }

    total
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
