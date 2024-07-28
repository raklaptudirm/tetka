use std::fmt::Display;
use std::ops::{Index, Not};
use std::str::FromStr;

use arrayvec::ArrayVec;

pub mod ataxx;
mod bitboard;
pub use bitboard::*;

/// Position is a generalized interface for board representations of a wide
/// range of games. It can be used to create game-agnostic software. Tetka
/// provides some of the popular board representations out of the box, but
/// custom ones can also be implemented by the library user.
pub trait Position:
    FromStr // FEN parsing support.
    + Display // Basic ascii display support.
    + Index<Self::ColoredPiece> // Support for fetching various BitBoards.
    + Index<<Self::ColoredPiece as ColoredPiece>::Piece>
    + Index<<Self::ColoredPiece as ColoredPiece>::Color>
where
    <Self as Index<Self::ColoredPiece>>::Output: Into<Self::BitBoard>,
    <Self as Index<<Self::ColoredPiece as ColoredPiece>::Piece>>::Output: Into<Self::BitBoard>,
    <Self as Index<<Self::ColoredPiece as ColoredPiece>::Color>>::Output: Into<Self::BitBoard>,
    Self::BitBoard: bitboard::BitBoard,
    Self::ColoredPiece: ColoredPiece,
    Self::Move: Move,
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
    fn insert(&mut self, sq: <Self::BitBoard as BitBoard>::Square, piece: <Self::ColoredPiece as ColoredPiece>::Piece);
    /// remove clears the given square, and returns the piece that was there.
    fn remove(&mut self, sq: <Self::BitBoard as BitBoard>::Square) -> Option<<Self::ColoredPiece as ColoredPiece>::Piece>;
    /// at returns the piece that is in the given square.
    fn at(&self, sq: <Self::BitBoard as BitBoard>::Square) -> Option<<Self::ColoredPiece as ColoredPiece>::Piece>;

    // Game Result functions.

    /// winner returns the winning side in the current position.
    fn winner(&self) -> Option<<Self::ColoredPiece as ColoredPiece>::Color>;
    /// is_game_over returns a boolean representing if the game is over.
    fn is_game_over(&self) -> bool { self.winner().is_some() }

    /// after_move returns the position after playing the given move on the
    /// current position. The UPDATE_PERIPHERALS flag can be interpreted as
    /// toggling the non-essential updated which are done by this function, like
    /// the hash function for the position.
    fn after_move<const UPDATE_PERIPHERALS: bool>(mov: Self::Move) -> Self;

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

/// The Move trait should be implemented the move representation of a game.
pub trait Move: FromStr + Display + From<u16> + Into<u16> {
    /// NULL represents the null or the 'do nothing' move.
    const NULL: Self;
    /// MAX_IN_GAME is a suitably high maximum for the number of move in a game.
    const MAX_IN_GAME: usize;
    /// MAX_IN_POSITION is a suitably high maximum for the number of move in a
    /// single, possibly unreachable position.
    const MAX_IN_POSITION: usize;
}

/// The ColoredPiece trait should be implemented by the piece representation
/// (with color) for a game.
pub trait ColoredPiece: RepresentableType<u8>
where
    Self::Piece: RepresentableType<u8>,
    Self::Color: RepresentableType<u8> + Not,
{
    /// Piece is the piece representation for the game.
    type Piece;
    /// Color is the color representation for the game.
    type Color;

    /// new creates a new ColoredPiece from the given Piece and Color.
    fn new(piece: Self::Piece, color: Self::Color) -> Self {
        Self::unsafe_from(color.into() * Self::Piece::N as u8 + piece.into())
    }

    /// piece returns the Piece part of the given ColoredPiece.
    fn piece(&self) -> Self::Piece;
    /// color returns the Color part of the given ColoredPiece.
    fn color(&self) -> Self::Color;
}

/// MoveStore is a trait implemented by types which are able to store moves
/// inside themselves and are thus usable in move-generation methods in
/// [Position] like [`Position::generate_moves_into<T>`].
pub trait MoveStore<M>: Default {
    /// push adds the given Move to the MoveStore.
    fn push(&mut self, m: M);

    /// len returns the number of [Move]s stored in the MoveStore.
    fn len(&self) -> usize;

    /// is_empty checks if no [Move]s are stored in the MoveStore.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// MoveList is a basic implementation of [`MoveStore`] that is used to allow users
/// to utilize move-generation methods without having to implement a [MoveStore] by
/// themselves. It also has utility methods other than the [`MoveStore`] trait.
pub type MoveList<M> = ArrayVec<M, 256>;

// MoveStore implementation for MoveList.
impl<M> MoveStore<M> for MoveList<M> {
    fn push(&mut self, m: M) {
        self.push(m);
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
