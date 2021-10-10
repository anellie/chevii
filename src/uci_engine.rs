use chess::{Board, ChessMove, Square};
use std::ffi::CStr;
use std::io::{Read, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::str::FromStr;
use std::thread;
use std::time::Duration;

const PONDER_TIME: u64 = 500;

pub struct UCIEngine {
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl UCIEngine {
    pub fn new_stockfish() -> Self {
        let mut sf = Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut engine = Self {
            stdin: sf.stdin.take().unwrap(),
            stdout: sf.stdout.take().unwrap(),
        };

        engine.init();
        engine
    }

    fn init(&mut self) {
        self.read();
        let output = self.send_command_read("uci");
        assert!(output.contains("uciok"));
        self.send_command("setoption name Threads value 4");
        self.send_command("setoption name Skill Level value 3");
    }

    fn send_command(&mut self, command: &str) {
        writeln!(self.stdin, "{}", command).unwrap();
    }

    fn send_command_read(&mut self, command: &str) -> String {
        writeln!(self.stdin, "{}", command).unwrap();
        self.read()
    }

    fn read(&mut self) -> String {
        let mut buf = [0u8; 32000];
        let len = self.stdout.read(&mut buf).unwrap();
        assert!(len < 32000);
        let string = CStr::from_bytes_with_nul(&buf[0..=len]).unwrap();
        string.to_str().unwrap().to_string()
    }

    pub fn ponder_best_move(&mut self, board: &Board) -> ChessMove {
        self.send_command(&format!("position fen {}", board));
        loop {
            let res = self.send_command_read("isready");
            if res.contains("readyok") {
                break;
            }
        }

        self.send_command("go");
        thread::sleep(Duration::from_millis(PONDER_TIME));
        self.send_command("stop");

        let res = loop {
            let res = self.read();
            if res.contains("bestmove") {
                break res;
            }
        };
        Self::parse_best(board, res)
    }

    fn parse_best(board: &Board, output: String) -> ChessMove {
        let move_san = output.lines().last().unwrap().split(" ").nth(1).unwrap();

        match ChessMove::from_san(board, move_san) {
            Ok(mov) => mov,

            Err(_) => {
                // Sometimes the SAN parser does not enjoy Stockfish output
                let src_idx = move_san.find::<&[char]>(&NUMS).unwrap() - 1;
                let dest_idx = move_san.rfind::<&[char]>(&NUMS).unwrap() - 1;

                let src = Square::from_str(&move_san[src_idx..=src_idx + 1]).unwrap();
                let dest = Square::from_str(&move_san[dest_idx..=dest_idx + 1]).unwrap();

                ChessMove::new(src, dest, None)
            }
        }
    }
}

const NUMS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
