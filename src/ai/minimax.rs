use crate::ai::{evaluation, RatedMove};
use chess::{Board, BoardStatus, ChessMove, Color};
use rayon::prelude::*;

const DEPTH: usize = 6;
const ENDGAME_DEPTH: usize = 8;
const ENDGAME_THRESH: u32 = 10;

const INF: isize = 999999999999;
const WIN: isize = 99999999;

struct Suggestion {
    play: RatedMove,
    expected_move: Option<ChessMove>
}

pub fn calculate_move(board: Board) -> ChessMove {
    let moves = evaluation::sorted_moves(&board);
    let depth = get_depth(&board);

    let res = calc_depth(board, depth, moves);

    if let Some(expected) = (res.0).expected_move {
        log::info!(
            "Playing {} (score {}), expecting {}",
            res.0.play.0,
            (res.0).play.1,
            expected
        );
    } else {
        log::info!(
            "Playing {} (score {}, checkmate)",
            res.0.play.0,
            (res.0).play.1,
        );
    }

    res.0.play.0
}

fn calc_depth(board: Board, depth: usize, mut moves: Vec<RatedMove>) -> (Suggestion, Vec<RatedMove>) {
    let res = moves
        .par_iter_mut()
        .map(|(mov, score)| {
            let clone = board.make_move_new(*mov);
            let (basic_score, expected) = minimax(&clone, depth - 1, board.side_to_move(), -INF, INF);
            *score = basic_score + evaluation::eval_move(&board, *mov);
            ((*score, expected), *mov)
        })
        .max_by_key(|(score, _)| score.0)
        .unwrap();

    moves.par_sort_unstable_by_key(|mov| -mov.1);
    (Suggestion {
        play: (res.1, (res.0).0),
        expected_move: (res.0).1
    }, moves)
}

fn get_depth(board: &Board) -> usize {
    if (board.color_combined(Color::White).popcnt() + board.color_combined(Color::Black).popcnt())
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
    player: Color,
    mut alpha: isize,
    mut beta: isize,
) -> (isize, Option<ChessMove>) {
    match board.status() {
        BoardStatus::Checkmate if board.side_to_move() == player => return (-WIN, None),
        BoardStatus::Checkmate => return (WIN + (WIN * depth as isize), None),
        BoardStatus::Stalemate => return (-WIN / 2, None),
        BoardStatus::Ongoing if depth == 0 => return (evaluation::eval_board(board), None),
        _ => (),
    }

    let moves = evaluation::sorted_moves(board);
    let mut tmp = board.clone();
    if board.side_to_move() == player {
        let mut max_score = -INF;
        let mut best_move = ChessMove::default();

        for (mov, _) in moves {
            board.make_move(mov, &mut tmp);
            let (score, _) = minimax(&tmp, depth - 1, player, alpha, beta);

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

        for (mov, _) in moves {
            board.make_move(mov, &mut tmp);
            let (score, _) = minimax(&tmp, depth - 1, player, alpha, beta);

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
