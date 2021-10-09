use chess::{Board, ChessMove, ALL_PIECES, NUM_PIECES, Color, MoveGen, BoardStatus};
use std::ops::{BitXor, BitAnd};
use rayon::prelude::*;

const PLAYER: Color = Color::Black;
const OPPONENT: Color = Color::White;

const DEPTH: usize = 4;

pub fn get_best_move(board: &Board) -> ChessMove {
    let mut gen = MoveGen::new_legal(board);
    let moves = gen.collect::<Vec<_>>();

    dbg!(moves.into_par_iter().map(|mov| {
        let clone = board.make_move_new(mov);
        (minimax(&clone, DEPTH - 1, -99999, 99999), mov)
    }).max_by_key(|(score, _)| *score).unwrap()).1
}

fn minimax(board: &Board, depth: usize, mut alpha: isize, mut beta: isize) -> (isize, Option<ChessMove>) {
    if depth == 0 {
        return (eval_board(board), None);
    }
    if board.status() == BoardStatus::Checkmate {
        return if board.side_to_move() == PLAYER { (-999999, None) } else { (999999, None) };
    }

    let mut gen = MoveGen::new_legal(board);
    let mut tmp = board.clone();
    if board.side_to_move() == PLAYER {
        let mut max = -99999;
        let mut best = ChessMove::default();

        for mov in gen {
            board.make_move(mov, &mut tmp);
            let (score, _) = minimax(&tmp, depth - 1, alpha, beta);

            alpha = isize::max(alpha, score);
            if beta <= alpha {
                break;
            }

            if score >= max {
                max = score;
                best = mov;
            }
        }

        (max, Some(best))
    } else {
        let mut min = 99999;
        let mut best = ChessMove::default();

        for mov in gen {
            board.make_move(mov, &mut tmp);
            let (score, _) = minimax(&tmp, depth - 1, alpha, beta);

            beta = isize::min(beta, score);
            if beta <= alpha {
                break;
            }

            if score <= min {
                min = score;
                best = mov;
            }
        }

        (min, Some(best))
    }
}

fn eval_board(board: &Board) -> isize {
    let mut total = 0;
    let max = board.color_combined(PLAYER);
    let min = board.color_combined(OPPONENT);

    for piece in ALL_PIECES {
        let value = PIECE_VALUE[piece.to_index()];
        let bits = board.pieces(piece);
        total += (max.bitand(bits).popcnt() * value) as isize;
        total -= (min.bitand(bits).popcnt() * value) as isize;
    }

    total
}

const PIECE_VALUE: [u32; NUM_PIECES] = [
    1,
    3,
    3,
    5,
    8,
    9999,
];

mod tests {
    use chess::Board;
    use crate::minimax::eval_board;

    #[test]
    fn new_board_is_0() {
        let board = Board::default();
        assert_eq!(eval_board(&board), 0);
    }
}




/*

fn find_best(board: &Board, multi: usize) -> ChessMove {
    let gen = MoveGen::new_legal(&board);
    let mut tmp = board.clone();

    let mut best_move = ChessMove::default();
    let mut best_value = -999;
    for mov in gen {
        board.make_move(mov, &mut tmp);
        let value = eval_board(&tmp) * multi;
        if value > best_value {
            best_move = mov;
            best_value = value;
        }
    }

    best_move
}*/