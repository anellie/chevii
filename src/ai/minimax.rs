use crate::ai;
use crate::ai::statistics::Stat;
use crate::ai::table::{Entry, TransTable};
use crate::ai::{evaluation, RatedMove};
use chess::{Board, ChessMove, EMPTY};
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

pub fn calculate_move_until_depth(board: Board, depth: isize) -> ChessMove {
    let table = TransTable::new();
    let mut moves = ai::sorted_moves(&board, &table);
    calc_depth(board, &table, depth, &mut moves);
    moves[0].0
}

fn run_until_stopped(board: Board, move_tx: Sender<ChessMove>, run: &AtomicBool) {
    let start_time = Instant::now();
    let mut depth = 1;
    let table = TransTable::new();
    let mut moves = ai::sorted_moves(&board, &table);

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
        Stat::next_depth();
    }
}

fn calc_depth(board: Board, table: &TransTable, depth: isize, moves: &mut Vec<RatedMove>) {
    moves.par_iter_mut().for_each(|(mov, score)| {
        let clone = board.make_move_new(*mov);
        *score = -minimax(&clone, table, depth - 1, -INF, INF);
    });
    moves.par_sort_unstable_by_key(|mov| -mov.1);
}

fn minimax(
    board: &Board,
    table: &TransTable,
    depth: isize,
    mut alpha: isize,
    beta: isize,
) -> isize {
    if depth == 0 {
        Stat::NodesEvaluated.inc();
        return explore_captures(board, table, alpha, beta);
    }

    let hash = board.get_hash();
    match table.get(hash) {
        Some(entry) if entry.depth >= depth as i32 => {
            Stat::TableHits.inc();
            return entry.score as isize;
        }
        _ => Stat::TableMisses.inc(),
    }

    let moves = ai::sorted_moves(board, &table);
    match moves.len() {
        0 if board.checkers() != &EMPTY => {
            // Lost
            Stat::CheckmatesFound.inc();
            return -(WIN + (depth * 1000) as isize);
        }
        0 => return -WIN / 2, // Stalemate
        _ => (),
    }

    let mut tmp = board.clone();
    for (mov, _) in moves {
        board.make_move(mov, &mut tmp);
        let score = -minimax(&tmp, table, depth - 1, -beta, -alpha);

        if score >= beta {
            table.put(Entry {
                zobrist: hash,
                score: score as i32,
                depth: depth as i32,
            });
            Stat::BranchesCut.inc();
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    table.put(Entry {
        zobrist: hash,
        score: alpha as i32,
        depth: depth as i32,
    });
    alpha
}

fn explore_captures(board: &Board, table: &TransTable, mut alpha: isize, beta: isize) -> isize {
    let score = evaluation::eval_board(board);
    if score >= beta {
        return beta;
    }
    if score > alpha {
        alpha = score;
    }

    let moves = ai::capturing_moves(board, &table);
    let mut tmp = board.clone();
    for (mov, _) in moves {
        board.make_move(mov, &mut tmp);
        let score = -explore_captures(&tmp, table, -beta, -alpha);

        if score >= beta {
            Stat::BranchesCut.inc();
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

#[cfg(test)]
mod tests {
    use super::calculate_move_until_depth;
    use chess::Board;
    use std::str::FromStr;
    use test::Bencher;

    #[bench]
    fn bench_depth_3(b: &mut Bencher) {
        let board =
            Board::from_str("r1bqk2r/ppp2pp1/2n2n2/3Pp2p/2P5/P2P1N2/2P2PPP/R1BQKB1R b KQkq - 0 8")
                .unwrap();
        b.iter(|| calculate_move_until_depth(board, 3));
    }
}
