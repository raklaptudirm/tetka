# <samp> ataxx </samp>

![Build Status](https://img.shields.io/github/actions/workflow/status/raklaptudirm/mexx/ci.yml) ![License](https://img.shields.io/crates/l/ataxx) ![Crates.io](https://img.shields.io/crates/v/ataxx
)

<samp>ataxx</samp> is a Rust package which provides various functionalities to deal with the [Ataxx](https://en.wikipedia.org/wiki/Ataxx) game in pure Rust. It provides various functionalities like board representation, move generation, UAI client creation, etc.

```rs
use std::str::FromStr;
use ataxx::{ Board, Square, Move };

fn main() {
    // Parse Ataxx Board from FEN
    let mut board = Board::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    println!("{}" board);

    // Make moves on the Board
    board.make_move(Move::new_single(Square::F1));
    println!("{}", board);

    // Undo moves from the Board
    board.undo_move();
    println!("{}", board);
}
```

## Features
- Fast Board representation and Move generation using BitBoards.
- Types for various features of Ataxx, including `Board`, `Position`, `Move`, `Square`, `Color`, etc.
- Support for semi-unique hashing of Ataxx positions for hash tables.
- Parsing `Position` and `Board` from FEN strings.

Refer to the [documentation](https://docs.rs/) for a full in depth list of features and functions.
