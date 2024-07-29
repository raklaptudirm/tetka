use std::fmt::Display;
use std::ops::Index;
use std::str::FromStr;

use super::{BitBoardType, ColoredPieceType, MoveList, MoveStore, MoveType};

/// Position is a generalized interface for board representations of a wide
/// range of games. It can be used to create game-agnostic software. Tetka
/// provides some of the popular board representations out of the box, but
/// custom ones can also be implemented by the library user.
pub trait PositionType:
    FromStr // FEN parsing support.
    + Display // Basic ascii display support.
    + Index<Self::ColoredPiece> // Support for fetching various BitBoards.
    + Index<<Self::ColoredPiece as ColoredPieceType>::Piece>
    + Index<<Self::ColoredPiece as ColoredPieceType>::Color>
where
    <Self as Index<Self::ColoredPiece>>::Output: Into<Self::BitBoard>,
    <Self as Index<<Self::ColoredPiece as ColoredPieceType>::Piece>>::Output: Into<Self::BitBoard>,
    <Self as Index<<Self::ColoredPiece as ColoredPieceType>::Color>>::Output: Into<Self::BitBoard>,
    Self::BitBoard: BitBoardType,
    Self::ColoredPiece: ColoredPieceType,
    Self::Move: MoveType,
{
    /// The type for the bitboards used by this board representation.
    type BitBoard;

    /// The type for the pieces (with color) used by this board representation.
    type ColoredPiece;

    /// The type for one move in this board representation.
    type Move;

    // Peeking, insertion, and removal of pieces from the board representation.

    /// insert puts the given piece at the given square. The implementation is
    /// free to assume that the square is currently empty.
    fn insert(&mut self, sq: <Self::BitBoard as BitBoardType>::Square, piece: Self::ColoredPiece);
    /// remove clears the given square, and returns the piece that was there.
    fn remove(&mut self, sq: <Self::BitBoard as BitBoardType>::Square) -> Option<Self::ColoredPiece>;
    /// at returns the piece that is in the given square.
    fn at(&self, sq: <Self::BitBoard as BitBoardType>::Square) -> Option<Self::ColoredPiece>;

    // Game Result functions.

    /// winner returns the winning side in the current position.
    fn winner(&self) -> Option<<Self::ColoredPiece as ColoredPieceType>::Color>;
    /// is_game_over returns a boolean representing if the game is over.
    fn is_game_over(&self) -> bool { self.winner().is_some() }

    /// after_move returns the position after playing the given move on the
    /// current position. The UPDATE_PERIPHERALS flag can be interpreted as
    /// toggling the non-essential updated which are done by this function, like
    /// the hash function for the position.
    fn after_move<const UPDATE_PERIPHERALS: bool>(&self, mov: Self::Move) -> Self;

    // Move Generation functions for the board representation.

    /// generate_moves_into generates all the moves in the current position into
    /// the given move storage. The QUIET and NOISY flags toggles the generation
    /// of reversible and irreversible moves respectively.
    fn generate_moves_into<
        const QUIET: bool, const NOISY: bool,
        T: MoveStore<Self::Move>
    >(&self, movelist: &mut T);
    /// generate_moves is similar to generate_moves_into, except that instead of
    /// taking some storage as input it stores into a custom stack-based type.
    fn generate_moves<const QUIET: bool, const NOISY: bool>(&self) -> MoveList<Self::Move> {
        let mut movelist: MoveList<Self::Move> = Default::default();
        self.generate_moves_into::<QUIET, NOISY, _>(&mut movelist);
        movelist
    }
    /// count_moves is similar to generate_moves, except instead of returning a
    /// list of the available moves, it returns the number of available moves.
    /// By default this is simply `generate_moves().len()`, but implementations
    /// may take advantage of various optimizations counting as opposed to
    /// storing the moves allows to provide a more efficient version.
    fn count_moves<const QUIET: bool, const NOISY: bool>(&self) -> usize {
        self.generate_moves::<QUIET, NOISY>().len()
    }
}
