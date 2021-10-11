use crate::ai::{evaluation, OPPONENT, PLAYER};
use chess::{Board, BoardStatus, ChessMove};
use rayon::prelude::*;

const DEPTH: usize = 6;
const ENDGAME_DEPTH: usize = 8;
const ENDGAME_THRESH: u32 = 10;

const INF: isize = 999999999999;
const WIN: isize = 99999999;

pub fn get_best_move(board: &Board) -> ChessMove {
    let moves = evaluation::sorted_moves(board);
    let depth = get_depth(board);

    let res = moves
        .into_par_iter()
        .map(|mov| {
            let clone = board.make_move_new(mov);
            (minimax(&clone, depth - 1, -INF, INF), mov)
        })
        .max_by_key(|(score, _)| *score)
        .unwrap();

    if let Some(expected) = (res.0).1 {
        log::info!(
            "Playing {} (score {}), expecting {}",
            res.1,
            (res.0).0,
            expected
        );
    } else {
        log::info!(
            "Playing {} (score {}, checkmate)",
            res.1,
            (res.0).0
        );
    }

    res.1
}

fn get_depth(board: &Board) -> usize {
    if (board.color_combined(PLAYER).popcnt() + board.color_combined(OPPONENT).popcnt())
        <= ENDGAME_THRESH
    {
        ENDGAME_DEPTH
    } else {
        DEPTH
    }
}

fn minimax(
    board: &Board,
    depth: usize,
    mut alpha: isize,
    mut beta: isize,
) -> (isize, Option<ChessMove>) {
    match board.status() {
        BoardStatus::Checkmate if board.side_to_move() == PLAYER => return (-WIN, None),
        BoardStatus::Checkmate => return (WIN, None),
        BoardStatus::Stalemate => return (-WIN / 2, None),
        BoardStatus::Ongoing if depth == 0 => return (evaluation::eval_board(board), None),
        _ => (),
    }

    let moves = evaluation::sorted_moves(board);
    let mut tmp = board.clone();
    if board.side_to_move() == PLAYER {
        let mut max_score = -INF;
        let mut best_move = ChessMove::default();

        for mov in moves {
            board.make_move(mov, &mut tmp);
            let (score, _) = minimax(&tmp, depth - 1, alpha, beta);

            max_score = isize::max(max_score, score);
            alpha = isize::max(alpha, max_score);
            if beta <= alpha {
                break;
            }

            if score >= max_score {
                max_score = score;
                best_move = mov;
            }
        }

        (max_score, Some(best_move))
    } else {
        let mut min_score = INF;
        let mut best_move = ChessMove::default();

        for mov in moves {
            board.make_move(mov, &mut tmp);
            let (score, _) = minimax(&tmp, depth - 1, alpha, beta);

            min_score = isize::min(min_score, score);
            beta = isize::min(beta, min_score);
            if beta <= alpha {
                break;
            }

            if score <= min_score {
                min_score = score;
                best_move = mov;
            }
        }

        (min_score, Some(best_move))
    }
}
