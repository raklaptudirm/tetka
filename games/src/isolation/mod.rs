// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bitboard::*;
pub use self::position::*;
pub use self::r#move::*;

// Non-namespaced modules.
mod bitboard;
mod r#move;
mod position;

#[cfg(test)]
mod tests;

crate::interface::game_details!(
    @bitboard_less
    Files: A, B, C, D, E, F, G, H;
    Ranks: 1 First, 2 Second, 3 Third, 4 Fourth, 5 Fifth, 6 Sixth;
    Pieces: Pawn "p"; Tile "-";
    Colors: White "w" ("P"),
            Black "b" ("p");
);
