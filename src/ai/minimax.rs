use crate::ai;
use crate::ai::statistics::Stat;
use crate::ai::table::{Entry, TransTable};
use crate::ai::{evaluation, RatedMove};
use chess::{Board, ChessMove, EMPTY};
use rayon::iter::Either;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

const INF: i32 = 999999999;
const WIN: i32 = 999999;

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

#[cfg(test)]
pub fn calculate_move_until_depth(board: Board, depth: i16) -> ChessMove {
    let table = TransTable::new();
    let mut moves = ai::sorted_moves(&board, &table);
    calc_depth(board, &table, depth, &mut moves);
    moves[0].0
}

fn run_until_stopped(board: Board, move_tx: Sender<ChessMove>, run: &AtomicBool) {
    let start_time = Instant::now();
    let mut depth = 2;
    let table = TransTable::new();
    let mut moves = ai::sorted_moves(&board, &table);

    while run.load(Ordering::Relaxed) {
        calc_depth(board, &table, depth, &mut moves);
        move_tx.send(moves[0].0).ok();
        log::info!(
            "Reached depth {} with {} moves in {}s",
            depth,
            moves.len(),
            start_time.elapsed().as_secs_f32()
        );
        log::debug!("Best Move: {}", moves[0].0);
        depth += 1;
        Stat::next_depth();
    }
}

fn calc_depth(board: Board, table: &TransTable, depth: i16, moves: &mut Vec<RatedMove>) {
    if depth >= 4 {
        moves.truncate(usize::max(5,moves.len() / 2));
    }
    moves.par_iter_mut().for_each(|(mov, score)| {
        let time = Instant::now();
        let clone = board.make_move_new(*mov);
        *score = -minimax(&clone, table, depth - 1, depth, -INF, INF);
        log::trace!("Spent {}s on move {} at depth {}", time.elapsed().as_secs_f32(), mov, depth);
    });
    moves.par_sort_unstable_by_key(|mov| -mov.1);
}

fn minimax(
    board: &Board,
    table: &TransTable,
    depth: i16,
    total_depth: i16,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    let (hash, moves) = match init_search(board, table, depth, alpha, beta) {
        Either::Left(score) => return score,
        Either::Right(moves) => moves,
    };

    let mut tmp = board.clone();
    for (mov, _) in &moves {
        board.make_move(*mov, &mut tmp);
        let score = if *mov == moves[0].0 {
            -minimax(&tmp, table, depth - 1, total_depth, -beta, -alpha)
        } else {
            let score = -scout_search(&tmp, table, depth - 1, -alpha);
            if alpha < score && score < beta {
                Stat::PVMisses.inc();
                -minimax(&tmp, table, depth - 1, total_depth, -beta, -score)
            } else {
                score
            }
        };

        if score >= beta {
            table.put(Entry {
                zobrist: hash,
                score: score as i32,
                depth_of_score: depth,
                depth_of_search: total_depth,
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
        depth_of_score: depth,
        depth_of_search: total_depth,
    });
    alpha
}

fn scout_search(board: &Board, table: &TransTable, depth: i16, beta: i32) -> i32 {
    let (_, moves) = match init_search(board, table, depth, beta - 1, beta) {
        Either::Left(score) => return score,
        Either::Right(moves) => moves,
    };

    let mut tmp = board.clone();
    for (mov, _) in &moves {
        board.make_move(*mov, &mut tmp);
        let score = -scout_search(&tmp, table, depth - 1, 1 - beta);
        if score >= beta {
            return beta;
        }
    }

    beta - 1
}

fn init_search(
    board: &Board,
    table: &TransTable,
    depth: i16,
    alpha: i32,
    beta: i32,
) -> Either<i32, (u64, Vec<RatedMove>)> {
    if depth == 0 {
        Stat::NodesEvaluated.inc();
        return Either::Left(explore_captures(board, table, alpha, beta));
    }

    let hash = board.get_hash();
    match table.get(hash) {
        Some(entry) if entry.depth_of_score >= depth => {
            Stat::TableHits.inc();
            return Either::Left(entry.score);
        }
        _ => Stat::TableMisses.inc(),
    }

    let moves = ai::sorted_moves(board, &table);
    match moves.len() {
        0 if board.checkers() != &EMPTY => {
            // Lost
            Stat::CheckmatesFound.inc();
            Either::Left(-(WIN + (depth as i32 * 1000)))
        }
        0 => Either::Left(-WIN / 2), // Stalemate
        _ => Either::Right((hash, moves)),
    }
}

fn explore_captures(board: &Board, table: &TransTable, mut alpha: i32, beta: i32) -> i32 {
    let score = evaluation::eval_board(board);
    if score >= beta {
        return beta;
    }
    if score > alpha {
        alpha = score;
    }

    let moves = ai::capturing_moves(board, &table);
    let mut tmp = board.clone();
    for (mov, _) in &moves {
        board.make_move(*mov, &mut tmp);
        let score = if *mov == moves[0].0 {
            -explore_captures(&tmp, table, -beta, -alpha)
        } else {
            let score = -explore_captures(&tmp, table, -alpha - 1, -alpha);
            if alpha < score && score < beta {
                Stat::PVMisses.inc();
                -explore_captures(&tmp, table, -beta, -score)
            } else {
                score
            }
        };

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
