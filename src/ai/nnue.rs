use chess::{BitBoard, Board, Color, Piece, NUM_PIECES};
use std::ffi::CString;

pub fn init() {
    let path = CString::new("model.nnue").unwrap();
    assert!(unsafe { probe::nnue_init(path.as_ptr()) })
}

pub fn eval(board: &Board) -> i32 {
    let mut index = 2;
    let mut pieces = [0i32; 33];
    let mut squares = [0i32; 33];

    let mut bb = board.color_combined(Color::White) | board.color_combined(Color::Black);
    while bb.0 != 0 {
        let sq = bb.to_square();
        let piece = board.piece_on(sq).unwrap();
        let piece_int = piece_to_idx(piece, board.color_on(sq).unwrap());
        if piece == Piece::King {
            let idx = board.color_on(sq).unwrap().to_index();
            pieces[idx] = piece_int as i32;
            squares[idx] = sq.to_index() as i32;
        } else {
            pieces[index] = piece_int as i32;
            squares[index] = sq.to_index() as i32;
            index += 1;
        }
        bb &= !BitBoard::from_square(sq);
    }

    let player = board.side_to_move().to_index() as i32;
    unsafe { probe::nnue_evaluate(player, pieces.as_mut_ptr(), squares.as_mut_ptr()) }
}

fn piece_to_idx(piece: Piece, color: Color) -> u32 {
    (NUM_PIECES - piece.to_index()) as u32 + ((color.to_index() as u32) * 6)
}

#[allow(unused)]
fn eval_fen(board: &Board) -> i32 {
    let fen = CString::new(board.to_string()).unwrap();
    unsafe { probe::nnue_evaluate_fen(fen.as_ptr()) }
}

#[cxx::bridge]
mod probe {
    extern "C++" {
        include!("chevii/src/ai/nnue/nnue.h");

        pub unsafe fn nnue_init(path: *const c_char) -> bool;
        pub unsafe fn nnue_evaluate(player: i32, pieces: *mut i32, squares: *mut i32) -> i32;
        pub unsafe fn nnue_evaluate_fen(fen: *const c_char) -> i32;
    }
}

#[cfg(test)]
mod tests {
    use super::{eval, piece_to_idx};
    use crate::ai::nnue::{eval_fen, init};
    use chess::{Board, Color, Piece};
    use std::str::FromStr;
    use test::Bencher;

    #[test]
    fn test_idx() {
        assert_eq!(piece_to_idx(Piece::King, Color::White), 1);
        assert_eq!(piece_to_idx(Piece::King, Color::Black), 7);
        assert_eq!(piece_to_idx(Piece::Queen, Color::White), 2);
        assert_eq!(piece_to_idx(Piece::Rook, Color::Black), 9);
    }

    #[test]
    fn test_identical() {
        init();
        let board =
            Board::from_str("r1bqk2r/ppp2pp1/2n2n2/3Pp2p/2P5/P2P1N2/2P2PPP/R1BQKB1R b KQkq - 0 8")
                .unwrap();
        assert_eq!(eval(&board), eval_fen(&board));
    }

    #[bench]
    fn bench_eval(b: &mut Bencher) {
        init();
        let board =
            Board::from_str("r1bqk2r/ppp2pp1/2n2n2/3Pp2p/2P5/P2P1N2/2P2PPP/R1BQKB1R b KQkq - 0 8")
                .unwrap();
        b.iter(|| eval(&board));
    }

    #[bench]
    fn bench_eval_fen(b: &mut Bencher) {
        init();
        let board =
            Board::from_str("r1bqk2r/ppp2pp1/2n2n2/3Pp2p/2P5/P2P1N2/2P2PPP/R1BQKB1R b KQkq - 0 8")
                .unwrap();
        b.iter(|| eval_fen(&board));
    }
}
