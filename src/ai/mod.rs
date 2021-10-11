mod evaluation;
mod minimax;

use chess::{BitBoard, Color, Rank};
pub use minimax::get_best_move;

const PLAYER: Color = Color::Black;
const PLAYER_BACK_RANK: Rank = Rank::Eighth;
#[allow(unused)]
const PLAYER_BACK_RANK_BITS: BitBoard =
    BitBoard(0b1111111100000000000000000000000000000000000000000000000000000000);
#[allow(unused)]
const PLAYER_PAWN_RANK: Rank = Rank::Seventh;
const PLAYER_PAWN_RANK_BITS: BitBoard =
    BitBoard(0b0000000011111111000000000000000000000000000000000000000000000000);

const OPPONENT: Color = Color::White;

#[cfg(test)]
mod tests {
    use crate::ai::{minimax, PLAYER_BACK_RANK_BITS, PLAYER_PAWN_RANK_BITS};
    use crate::sq;
    use chess::{Board, ChessMove, Game, Piece};
    use test::Bencher;

    #[test]
    fn check_bits() {
        let board = Board::default();
        assert_eq!(
            (board.pieces(Piece::Pawn) & PLAYER_PAWN_RANK_BITS).popcnt(),
            8
        );
        assert_eq!(
            (board.pieces(Piece::Rook) & PLAYER_BACK_RANK_BITS).popcnt(),
            2
        );
        assert_eq!(
            (board.pieces(Piece::Bishop) & PLAYER_BACK_RANK_BITS).popcnt(),
            2
        );
        assert_eq!(
            (board.pieces(Piece::Queen) & PLAYER_BACK_RANK_BITS).popcnt(),
            1
        );
    }

    #[bench]
    fn first_move(b: &mut Bencher) {
        let mut game = Game::new();
        game.make_move(ChessMove::new(sq(52), sq(36), None));
        let board = game.current_position();
        b.iter(|| minimax::get_best_move(&board));
    }
}
