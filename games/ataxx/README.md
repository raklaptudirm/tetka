# <samp> ataxx </samp>

![Build Status](https://img.shields.io/github/actions/workflow/status/raklaptudirm/mexx/ci.yml) ![License](https://img.shields.io/crates/l/ataxx) ![Crates.io](https://img.shields.io/crates/v/ataxx
)

<samp>ataxx</samp> is a Rust package which provides various functionalities to deal with the [Ataxx](https://en.wikipedia.org/wiki/Ataxx) game in pure Rust. It provides various functionalities like board representation, move generation, UAI client creation, etc.

```rust
use ataxx::*;
use std::str::FromStr;

fn main() {
    let position = Position::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();

    println!("{}\n", position);
    println!("perft(6): {}", perft(position, 6));
}

// An implementation of the standard perft function which walks the move
// generation tree of strictly legal moves to count all the leaf nodes of a
// certain depth. Nodes are only counted at the end after the last make-move.
// Thus "higher" terminal nodes (e.g. mate or stalemate) are not counted,
// instead the number of move paths of a certain depth. Perft ignores draws by
// repetition or by the fifty-move rule.
//
// A more advanced implementation of perft is available as `ataxx::perft`.
fn perft(position: Position, depth: u8) -> u64 {
    // perft(0) = 1
    if depth == 0 {
        return 1;
    }

    // perft(1) = number of moves in the position
    if depth == 1 {
        // Counting the number of moves in a Position is provided as a separate
        // function which is faster than the simple `generate_moves.len()`.
        return position.count_moves() as u64;
    }

    let mut nodes: u64 = 0;
    let movelist = position.generate_moves();

    // MoveList implements IntoIterator, so it can be directly used in a for loop.
    for m in movelist {
        // Providing false as the value of the `UPDATE_HASH` template constant
        // disables updating the position's Hash in `after_move` which makes
        // it faster than `after_move::<true>()` and can be done here since we
        // don't use the position's Hash anywhere.
        let new_position = position.after_move::<false>(m);
        nodes += perft(new_position, depth - 1);
    }

    nodes
}

```

## Features
- Fast Board representation and Move generation, resulting a perft faster than [<samp>libataxx</samp>](https://github.com/kz04px/libataxx).
- Types for various features of Ataxx, including `Position`, `Move`, `Square`, `Piece`, etc.
- Support for semi-unique hashing of Ataxx positions for hash tables, which is faster yet as strong as zobrist hashing.
- Parsing a FEN string into a `Position`.

Refer to the [documentation](https://docs.rs/ataxx) for a full in depth list of features and functions.
