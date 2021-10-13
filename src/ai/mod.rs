mod evaluation;
mod minimax;
pub mod book;

use chess::{BitBoard, Color, Rank, Board, ChessMove};

pub fn get_best_move(board: &Board) -> ChessMove {
    if let Some(mov) = book::get_for(board) {
        mov
    } else {
        minimax::calculate_move(board)
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

#[cfg(test)]
mod tests {
    use crate::ai::{minimax, PLAYER_BACK_RANK_BITS, PLAYER_PAWN_RANK_BITS};
    use crate::sq;
    use chess::{Board, ChessMove, Game, Piece};
    use test::Bencher;

    #[bench]
    fn first_move(b: &mut Bencher) {
        let mut game = Game::new();
        game.make_move(ChessMove::new(sq(52), sq(36), None));
        let board = game.current_position();
        b.iter(|| minimax::get_best_move(&board));
    }
}
