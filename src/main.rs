#![feature(test)]

extern crate test;

use chess::Board;
use rayon::ThreadPoolBuilder;
use std::process;
use std::str::FromStr;
use structopt::StructOpt;

pub mod ai;

#[derive(StructOpt, Debug)]
struct Opt {
    /// Produce a single move
    #[structopt(short, long)]
    position: String,

    /// Number of threads to use
    #[structopt(long, default_value = "8")]
    threads: usize,

    /// Time for thinking per move
    #[structopt(short, long, default_value = "3")]
    time: f32,
}

fn main() {
    env_logger::init();
    let opts = Opt::from_args();
    ThreadPoolBuilder::new()
        .num_threads(opts.threads)
        .build_global()
        .unwrap();

    let board = Board::from_str(&opts.position).unwrap();
    let mov = ai::get_best_move(board, opts.time);
    println!("{}", mov);
    process::exit(0);
}
