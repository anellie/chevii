use std::fs::{read_to_string, File};
use chess::{Game, ChessMove, Color, Board};
use std::collections::HashMap;
use std::io::Write;
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};

const BOOK_STR: &str = include_str!("../../book");
lazy_static! {
    static ref BOOK: HashMap<(u64, Color), (Vec<(usize, String)>, usize)> = make_book();
}

pub fn get_for(board: &Board) -> Option<ChessMove> {
    let hash = board.get_hash();
    let movs = &BOOK.get(&(hash, board.side_to_move()))?;

    let mut rand = thread_rng().gen_range(0..movs.1) as isize;
    let mut index = 0;
    while rand > 0 {
        rand -= movs.0[index].0 as isize;
        index += 1;
    }

    let mov_str = &movs.0[index].1;
    Some(ChessMove::from_san(board, mov_str).ok()?)
}

fn make_book() -> HashMap<(u64, Color), (Vec<(usize, String)>, usize)> {
    let mut out = HashMap::new();
    for position in BOOK_STR.lines() {
        let mut iter = position.split("   ");
        let hash = u64::from_str_radix(iter.next().unwrap(), 16).unwrap();
        let color = if iter.next().unwrap() == "w" {
            Color::White
        } else {
            Color::Black
        };
        let moves = iter.next().unwrap();
        let mut total = 0;
        let mut movs = Vec::new();
        for mov in moves.split(" ") {
            let mut split = mov.split(":");
            let count = usize::from_str_radix(split.next().unwrap(), 10).unwrap();
            total += count;
            let mov = split.next().unwrap();
            movs.push((count, mov.to_string()));
        }
        out.insert((hash, color), (movs, total));
    }
    out
}

// Convert from a weird pgn mod where games are delimited by 2 newlines
#[allow(unused)]
pub fn convert_pgn() {
    let book = read_to_string("games.pgn").unwrap();
    let mut out = HashMap::<Board, HashMap<ChessMove, usize>>::new();
    for game in book.split("\n\n").take(1000000) {
        let write = game
            .split_whitespace()
            .filter(|s| !s.ends_with(".") && !s.contains("0") && !s.contains("1/2"))
            .take(10);

        let mut g = Game::new();
        for mov in write {
            let board = g.current_position();
            let mov = ChessMove::from_san(&board, mov);
            let mov = if let Ok(mov) = mov {
                mov
            } else {
                break;
            };

            if let Some(vec) = out.get_mut(&board) {
                if let Some(count) = vec.get_mut(&mov) {
                    *count += 1
                } else {
                    vec.insert(mov, 1);
                }
            } else {
                let mut map = HashMap::new();
                map.insert(mov, 1);
                out.insert(board, map);
            };
            g.make_move(mov);
        }
    }
    println!("{}", out.len());

    let mut output = File::create("book").unwrap();
    for line in out {
        let hash = line.0.get_hash();
        let side = if line.0.side_to_move() == Color::White {
            "w"
        } else {
            "b"
        };
        write!(output, "{:x}   {}  ", hash, side).unwrap();
        for mov in line.1 {
            write!(output, " {}:{}", mov.1, mov.0).unwrap();
        }
        writeln!(output).unwrap();
    }
}
