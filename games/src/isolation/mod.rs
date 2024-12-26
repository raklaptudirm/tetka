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
    Files: A, B, C, D, E, F, G, H;
    Ranks: 1 First, 2 Second, 3 Third, 4 Fourth, 5 Fifth, 6 Sixth;
    Pieces: Pawn "p"; Tile "-";
    Colors: White "w" ("P"),
            Black "b" ("p");
);

use crate::interface::BitBoardType;

impl BitBoard {
    /// singles returns the targets of all singular moves from all the source
    /// squares given in the provided BitBoard.
    pub fn singles(bb: BitBoard) -> BitBoard {
        let bar = bb | bb.east() | bb.west();
        (bar | bar.north() | bar.south()) ^ bb
    }
}
