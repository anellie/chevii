use crate::ai::{evaluation, RatedMove};
use chess::{Board, BoardStatus, ChessMove, Color};
use rayon::prelude::*;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;

const START_DEPTH: isize = 4;

const INF: isize = 999999999999;
const WIN: isize = 99999999;

struct Suggestion {
    play: RatedMove,
    expected_move: Option<ChessMove>
}

pub fn calculate_move(board: Board, time: f32) -> ChessMove {
    let mut suggest = Suggestion {
        play: (Default::default(), 0),
        expected_move: None
    };
    let mut moves = evaluation::sorted_moves(&board);

    for iter_depth in 1..START_DEPTH {
        let (s, m) = calc_depth(board, iter_depth, moves);
        suggest = s;
        moves = m;
    }

    let (suggest_tx, suggest_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();
    let start_time = Instant::now();

    thread::spawn(move || {
        let mut depth = START_DEPTH;
        while let Ok(false) = stop_rx.recv() {
            let (s, m) = calc_depth(board, depth, moves);
            moves = m;
            suggest_tx.send(s).ok();
            depth += 1;
        }
        log::info!("Reached depth {} with {} moves", depth, moves.len());
    });

    stop_tx.send(false).ok();
    while start_time.elapsed().as_secs_f32() < time {
        if let Ok(value) = suggest_rx.try_recv() {
            stop_tx.send(false).ok();
            suggest = value;
        }
        thread::sleep(Duration::from_millis(1));
    }
    stop_tx.send(true).ok();
    drop(stop_tx);

    if let Some(expected) = suggest.expected_move {
        log::info!(
            "Playing {} (score {}), expecting {}",
            suggest.play.0,
            suggest.play.1,
            expected
        );
    } else {
        log::info!(
            "Playing {} (score {}, checkmate)",
            suggest.play.0,
            suggest.play.1,
        );
    }

    suggest.play.0
}

fn calc_depth(board: Board, depth: isize, mut moves: Vec<RatedMove>) -> (Suggestion, Vec<RatedMove>) {
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

fn minimax(
    board: &Board,
    depth: isize,
    player: Color,
    mut alpha: isize,
    mut beta: isize,
) -> (isize, Option<ChessMove>) {
    let moves = match board.status() {
        BoardStatus::Checkmate if board.side_to_move() == player => return (-(WIN + (depth * 1000) as isize), None),
        BoardStatus::Checkmate => return (WIN + (WIN * depth as isize), None),
        BoardStatus::Stalemate => return (-WIN / 2, None),

        BoardStatus::Ongoing if depth <= 0 => {
            let capture_moves = evaluation::capturing_moves(board);
            if !capture_moves.is_empty() {
                capture_moves
            } else {
                return (evaluation::eval_board(board, player), None);
            }
        },

        _ => evaluation::sorted_moves(board),
    };

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
