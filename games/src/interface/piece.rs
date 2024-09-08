use std::ops::Not;

use super::RepresentableType;

/// The ColoredPiece trait should be implemented by the piece representation
/// (with color) for a game.
pub trait ColoredPieceType: RepresentableType<u8>
where
    Self::Piece: RepresentableType<u8>,
    Self::Color: RepresentableType<u8> + Not,
{
    /// Piece is the piece representation for the game.
    type Piece;
    /// Color is the color representation for the game.
    type Color;

    /// new creates a new ColoredPiece from the given Piece and Color.
    #[must_use]
    fn new(piece: Self::Piece, color: Self::Color) -> Self {
        unsafe { Self::unsafe_from(color.into() * Self::Piece::N as u8 + piece.into()) }
    }

    /// piece returns the Piece part of the given ColoredPiece.
    #[must_use]
    fn piece(self) -> Self::Piece;
    /// color returns the Color part of the given ColoredPiece.
    #[must_use]
    fn color(self) -> Self::Color;
}
