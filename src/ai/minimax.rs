use crate::ai;
use crate::ai::table::{Entry, TransTable};
use crate::ai::{evaluation, RatedMove};
use chess::{Board, BoardStatus, ChessMove, Color, EMPTY};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

const INF: isize = 999999999999;
const WIN: isize = 99999999;

pub fn calculate_move(board: Board, time: f32) -> ChessMove {
    let mut cmove = ChessMove::default();
    let (move_tx, move_rx) = mpsc::channel();
    let run = AtomicBool::new(true);
    let start_time = Instant::now();

    thread::spawn(move || run_until_stopped(board, move_tx, &run));

    while start_time.elapsed().as_secs_f32() < time {
        if let Ok(value) = move_rx.try_recv() {
            cmove = value;
        }
        thread::sleep(Duration::from_millis(1));
    }

    cmove
}

fn run_until_stopped(board: Board, move_tx: Sender<ChessMove>, run: &AtomicBool) {
    let start_time = Instant::now();
    let mut depth = 1;
    let mut moves = ai::sorted_moves(&board);
    let table = TransTable::new();

    while run.load(Ordering::Relaxed) {
        calc_depth(board, &table, depth, &mut moves);
        move_tx.send(moves[0].0).ok();
        depth += 1;
        log::info!(
            "Reached depth {} with {} moves in {}s",
            depth,
            moves.len(),
            start_time.elapsed().as_secs_f32()
        );
        log::debug!("Best Move: {}", moves[0].0);
    }
}

fn calc_depth(board: Board, table: &TransTable, depth: isize, moves: &mut Vec<RatedMove>) {
    moves.par_iter_mut().for_each(|(mov, score)| {
        let clone = board.make_move_new(*mov);
        *score = minimax(&clone, table, depth - 1, board.side_to_move(), -INF, INF);
    });
    moves.par_sort_unstable_by_key(|mov| -mov.1);
}

fn minimax(
    board: &Board,
    table: &TransTable,
    depth: isize,
    player: Color,
    mut alpha: isize,
    mut beta: isize,
) -> isize {
    let hash = board.get_hash();
    match table.get(hash) {
        Some(entry) if entry.depth >= depth as i32 => return entry.score as isize,
        _ => (),
    }

    // Kinda ugly but allows saving the expensive `board.status()` call on non-capture calls
    let moves = if depth <= 0 {
        let moves = ai::capturing_moves(board);
        match board.status() {
            BoardStatus::Checkmate if board.side_to_move() == player => {
                return -(WIN + ((depth + 1000) * 1000) as isize)
            }
            BoardStatus::Checkmate => return WIN + (WIN * (depth + 1000) as isize),
            BoardStatus::Stalemate => return -WIN / 2,
            BoardStatus::Ongoing if moves.len() == 0 => {
                return evaluation::eval_board(board, player)
            }
            _ => moves,
        }
    } else {
        let moves = ai::sorted_moves(board);
        match moves.len() {
            0 if board.checkers() != &EMPTY && board.side_to_move() == player => {
                return -(WIN + (depth * 1000) as isize)
            } // Lost
            0 if board.checkers() != &EMPTY => return WIN + (WIN * depth as isize), // Won
            0 => return -WIN / 2,                                                   // Stalemate
            _ => moves,
        }
    };

    let mut tmp = board.clone();
    let score = if board.side_to_move() == player {
        let mut max_score = -INF;

        for (mov, _) in moves {
            board.make_move(mov, &mut tmp);
            let score = minimax(&tmp, table, depth - 1, player, alpha, beta);

            max_score = isize::max(max_score, score);
            alpha = isize::max(alpha, max_score);
            if beta <= alpha {
                break;
            }
        }

        max_score
    } else {
        let mut min_score = INF;

        for (mov, _) in moves {
            board.make_move(mov, &mut tmp);
            let score = minimax(&tmp, table, depth - 1, player, alpha, beta);

            min_score = isize::min(min_score, score);
            beta = isize::min(beta, min_score);
            if beta <= alpha {
                break;
            }
        }

        min_score
    };

    table.put(Entry {
        zobrist: hash,
        score: score as i32,
        depth: depth as i32,
    });
    score
}
