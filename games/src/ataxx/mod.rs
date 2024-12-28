use std::sync::LazyLock;

use strum::IntoEnumIterator;

// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::position::*;
pub use self::r#move::*;

// Non-namespaced modules.
mod r#move;
mod position;

#[cfg(test)]
mod tests;

crate::interface::game_details!(
    Files: A, B, C, D, E, F, G;
    Ranks: 1 First, 2 Second, 3 Third, 4 Fourth, 5 Fifth, 6 Sixth, 7 Seventh;
    Pieces: Piece "x"; Block "-";
    Colors: Black "x" ("x"),
            White "o" ("o");
);

use crate::interface::SetType;
use crate::interface::{BitBoardType, RepresentableType};

impl BitBoard {
    /// singles returns the targets of all singular moves from all the source
    /// squares given in the provided BitBoard.
    pub fn singles(bb: BitBoard) -> BitBoard {
        let bar = bb | bb.east() | bb.west();
        bar | bar.north() | bar.south()
    }

    /// single returns the targets of a singular Move from the given Square.
    pub fn single(square: Square) -> BitBoard {
        SINGLES[square as usize]
    }

    /// double returns the targets of a jump Move from the given Square.
    pub fn double(square: Square) -> BitBoard {
        DOUBLES[square as usize]
    }
}

static SINGLES: LazyLock<[BitBoard; Square::N]> = LazyLock::new(|| {
    let mut singles = [BitBoard::EMPTY; Square::N];
    for square in Square::iter() {
        let square_bb = BitBoard::from(square);
        singles[square as usize] = BitBoard::singles(square_bb) ^ square_bb;
    }
    singles
});

static DOUBLES: LazyLock<[BitBoard; Square::N]> = LazyLock::new(|| {
    let mut doubles = [BitBoard::EMPTY; Square::N];
    for square in Square::iter() {
        let singles = BitBoard::singles(BitBoard::from(square));
        doubles[square as usize] = BitBoard::singles(singles) ^ singles;
    }
    doubles
});
