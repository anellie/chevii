use crate::ai::{get_player_back_rank, get_player_pawn_bits, RatedMove};
use chess::CastleRights::NoRights;
use chess::{
    BitBoard, Board, CastleRights, ChessMove, Color, MoveGen, Piece, Square, ALL_PIECES, NUM_PIECES,
};
use rand::{thread_rng, Rng};
use rayon::prelude::ParallelSliceMut;

const PIECE_VALUE: [u32; NUM_PIECES] = [100, 300, 300, 500, 900, 99900];
const CONSIDER_VALUE: [u32; NUM_PIECES] = [20, 60, 60, 100, 250, 9990];

const CASTLE_BONUS: i32 = 8;
const CHECK_PENALTY: i32 = 15;

pub(super) fn eval_board(board: &Board) -> isize {
    let player = board.side_to_move();
    let player_eval = eval_all(board, player);
    let opponent_eval = eval_all(board, !player);
    (player_eval - opponent_eval) as isize
}

fn eval_all(board: &Board, player: Color) -> i32 {
    eval_material(board, player)
        + eval_castling(board, player)
        + eval_king(board, player)
        + eval_bishop(board, player)
        + eval_pawns(board, player)
}

fn eval_material(board: &Board, player: Color) -> i32 {
    let mut total = 0;
    let max = board.color_combined(player);

    for piece in ALL_PIECES {
        let value = piece_value(piece) as u32;
        let bits = board.pieces(piece);
        total += ((max & bits).popcnt() * value) as i32;
    }

    total
}

fn eval_castling(board: &Board, player: Color) -> i32 {
    match board.castle_rights(player) {
        NoRights => 0,
        CastleRights::KingSide | CastleRights::QueenSide => CASTLE_BONUS,
        CastleRights::Both => CASTLE_BONUS * 2,
    }
}

fn eval_king(board: &Board, player: Color) -> i32 {
    let mut score = 0;
    if board.side_to_move() == player && board.checkers().popcnt() != 0 {
        score -= CHECK_PENALTY;
    }

    let bb_around_king = get_king_adjacent_squares(board.king_square(player));
    let pieces_around_king = bb_around_king & board.color_combined(player);
    score += (pieces_around_king.popcnt() * 8) as i32;

    score
}

fn eval_bishop(board: &Board, player: Color) -> i32 {
    ((board.color_combined(player) & board.pieces(Piece::Bishop)).popcnt() > 1) as i32 * 20
}

fn eval_pawns(board: &Board, player: Color) -> i32 {
    let score = 0;
    score
}

pub(super) fn eval_move(board: &Board, cmove: ChessMove) -> isize {
    let mut value = 0;
    let moving_piece = board.piece_on(cmove.get_source()).unwrap();
    let captured_piece = board.piece_on(cmove.get_dest());

    // Promoting is often good
    if let Some(promoted) = cmove.get_promotion() {
        value += consider_value(promoted);
    }

    // Capture highest-value opponent pieces with lowest-value pieces first
    if let Some(captured_piece) = captured_piece {
        value += (2 * consider_value(captured_piece)) - consider_value(moving_piece);
    }

    let undeveloped_pawns_count = (board.color_combined(board.side_to_move())
        & board.pieces(Piece::Pawn)
        & get_player_pawn_bits(board))
    .popcnt();
    let is_early_game = undeveloped_pawns_count >= 6;
    // Prioritize developing pawns earlygame
    if is_early_game && moving_piece == Piece::Pawn {
        let file = cmove.get_source().get_file().to_index();
        // Middle pawns first
        value += (8 - isize::abs(file as isize - 4)) * 10;
    }

    // Penalize moving a piece to the back rank to prevent 'undeveloping' pieces during earlygame
    if is_early_game && cmove.get_dest().get_rank() == get_player_back_rank(board) {
        value -= 35;
    }

    // Penalize developing the queen too early
    if undeveloped_pawns_count > 6 && moving_piece == Piece::Queen {
        value -= 25;
    }

    // Penalize moving the king when castling is still possible
    if moving_piece == Piece::King && board.my_castle_rights() != NoRights {
        value -= 100;
    }
    // Penalize moving the king out of the back rank
    if moving_piece == Piece::King && cmove.get_source().get_rank() == get_player_back_rank(board) {
        value -= 25;
    }

    // Introduce some random variation to prevent repetition (AI chooses first move if multiple 'ideal' moves found)
    value + thread_rng().gen_range(0..5)
}

fn piece_value(piece: Piece) -> isize {
    PIECE_VALUE[piece.to_index()] as isize
}

fn consider_value(piece: Piece) -> isize {
    CONSIDER_VALUE[piece.to_index()] as isize
}

fn get_king_adjacent_squares(pos: Square) -> BitBoard {
    let s = pos.left().unwrap_or_else(|| pos.right().unwrap());
    BitBoard::from_square(pos.left().unwrap_or(s))
        & BitBoard::from_square(pos.right().unwrap_or(s))
        & BitBoard::from_square(pos.up().unwrap_or(s))
        & BitBoard::from_square(pos.down().unwrap_or(s))
        & BitBoard::from_square(pos.left().map(|s| s.up()).flatten().unwrap_or(s))
        & BitBoard::from_square(pos.right().map(|s| s.down()).flatten().unwrap_or(s))
        & BitBoard::from_square(pos.up().map(|s| s.right()).flatten().unwrap_or(s))
        & BitBoard::from_square(pos.down().map(|s| s.left()).flatten().unwrap_or(s))
}

#[cfg(test)]
mod tests {
    use super::eval_board;
    use chess::{Board, Color};

    #[test]
    fn new_board_is_0() {
        let board = Board::default();
        assert_eq!(eval_board(&board, Color::Black), 0);
    }
}
