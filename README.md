# chevii
A chess AI written in Rust. 

## Features
- Based on the [`chess` crate](https://lib.rs/chess) (Board representation and move generation)
- Negamax with AB pruning
- PVS with zero window search
- Quiescence Search
- Iterative Deepening
- Transposition Table
- Stockfish NNUE networks for evaluation
- Multithreaded evaluation using `rayon`

## Build & Deploy
chevii is intended be deployed as a bot on lichess. Check out the instructions for using a "homemade bot" on [here](https://github.com/ShailChoksi/lichess-bot),
the `strategies.py` and `config.yml` needed are provided in this repo.

## Credit / Thank You
- Thank you to Daniel Shawul ([dshawul](https://github.com/dshawul)) for nnue-probe, which chevii uses to interface with Stockfish NNUE.
- Thank you to Jordan ([jordanbray](https://github.com/jordanbray)) and contributors for the chess crate.
