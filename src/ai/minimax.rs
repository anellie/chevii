use crate::ai::{evaluation, RatedMove};
use chess::{Board, BoardStatus, ChessMove, Color, EMPTY};
use rayon::prelude::*;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use crate::ai::table::{TransTable, Entry};

const INF: isize = 999999999999;
const WIN: isize = 99999999;

pub fn calculate_move(board: Board, time: f32) -> ChessMove {
    let mut play = (Default::default(), 0);
    let table = TransTable::new();
    let mut moves = evaluation::sorted_moves(&board);

    let (suggest_tx, suggest_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();
    let start_time = Instant::now();

    thread::spawn(move || {
        let mut depth = 1;
        while let Ok(false) = stop_rx.recv() {
            let (s, m) = calc_depth(board, &table, depth, moves);
            moves = m;
            suggest_tx.send(s).ok();
            depth += 1;
            log::info!("Reached depth {} with {} moves in {}s", depth, moves.len(), start_time.elapsed().as_secs_f32());
        }
    });

    stop_tx.send(false).ok();
    while start_time.elapsed().as_secs_f32() < time {
        if let Ok(value) = suggest_rx.try_recv() {
            stop_tx.send(false).ok();
            play = value;
        }
        thread::sleep(Duration::from_millis(1));
    }
    stop_tx.send(true).ok();
    drop(stop_tx);

    play.0
}

fn calc_depth(board: Board, table: &TransTable, depth: isize, mut moves: Vec<RatedMove>) -> (RatedMove, Vec<RatedMove>) {
    let res = moves
        .par_iter_mut()
        .map(|(mov, score)| {
            let clone = board.make_move_new(*mov);
            let basic_score = minimax(&clone, table, depth - 1, board.side_to_move(), -INF, INF);
            *score = basic_score + evaluation::eval_move(&board, *mov);
            (*score, *mov)
        })
        .max_by_key(|(score, _)| *score)
        .unwrap();

    moves.par_sort_unstable_by_key(|mov| -mov.1);
    ((res.1, res.0), moves)
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
        let moves = evaluation::capturing_moves(board);
        match board.status() {
            BoardStatus::Checkmate if board.side_to_move() == player => return -(WIN + (depth * 1000) as isize),
            BoardStatus::Checkmate => return WIN + (WIN * depth as isize),
            BoardStatus::Stalemate => return -WIN / 2,
            BoardStatus::Ongoing if moves.len() == 0 => return evaluation::eval_board(board, player),
            _ => moves,
        }
    } else {
        let moves = evaluation::sorted_moves(board);
        match moves.len() {
            0 if board.checkers() != &EMPTY && board.side_to_move() == player => return -(WIN + (depth * 1000) as isize), // Lost
            0 if board.checkers() != &EMPTY => return WIN + (WIN * depth as isize), // Won
            0 => return -WIN / 2, // Stalemate
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
        depth: depth as i32
    });
    score
}
