use std::fmt::Display;
use std::str::FromStr;

use super::{
    BitBoardType, Color, ColoredPieceType, Hash, MoveList, MoveStore, MoveType,
    Piece, Square,
};

/// Position is a generalized interface for board representations of a wide
/// range of games. It can be used to create game-agnostic software. Tetka
/// provides some of the popular board representations out of the box, but
/// custom ones can also be implemented by the library user.
pub trait PositionType: FromStr + Display
where
    Self::BitBoard: BitBoardType,
    Self::ColoredPiece: ColoredPieceType,
    Self::Move: MoveType,
{
    /// Type for the bitboards used by this board representation.
    type BitBoard;

    /// Type for the pieces (with color) used by this board representation.
    type ColoredPiece;

    /// Type for one move in this board representation.
    type Move;

    // Peeking, insertion, and removal of pieces from the board representation.

    /// Adds the given Piece to the given Square. If the target Square is
    /// non-empty, the behavior is undefined.
    fn insert(&mut self, sq: Square<Self>, piece: Self::ColoredPiece);
    /// Removes any Piece on the given Square, and returns the removed Piece.
    fn remove(&mut self, sq: Square<Self>) -> Option<Self::ColoredPiece>;
    /// Returns the Piece present at the given Square.
    #[must_use]
    fn at(
        &self,
        sq: <Self::BitBoard as BitBoardType>::Square,
    ) -> Option<Self::ColoredPiece>;

    /// Returns a BitBoard with all the Squares containing the given Piece.
    #[must_use]
    fn piece_bb(&self, piece: Piece<Self>) -> Self::BitBoard;
    /// Returns a BitBoard with all the Squares containing the given Color.
    #[must_use]
    fn color_bb(&self, color: Color<Self>) -> Self::BitBoard;
    /// Returns a BitBoard with all the Squares containing the given ColoredPiece.
    #[must_use]
    fn colored_piece_bb(&self, piece: Self::ColoredPiece) -> Self::BitBoard;

    fn side_to_move(&self) -> Color<Self>;
    fn half_move_clock(&self) -> usize;
    fn ply_count(&self) -> usize;
    /// Returns a semi-unique checksum of the current Position.
    #[must_use]
    fn hash(&self) -> Hash;

    // Game Result functions.

    /// Returns the side which has won in the current position, if any.
    #[must_use]
    fn winner(&self) -> Option<Color<Self>>;
    /// Returns `true` if the game is over in the current position.
    #[must_use]
    fn is_game_over(&self) -> bool {
        self.winner().is_some()
    }

    /// Returns the position which is reached after playing the given move on
    /// the current position.
    ///
    /// The `UPDATE_PERIPHERALS` flag can be interpreted as toggling the
    /// non-essential updated which are done by this function, like the hash
    /// function for the position.
    #[must_use]
    fn after_move<const UPDATE_PERIPHERALS: bool>(
        &self,
        mov: Self::Move,
    ) -> Self;

    // Move Generation functions for the board representation.

    /// Generates all the moves in the current position and add them into the
    ///  given move storage.
    ///
    /// The `ALLOW_ILLEGAL` flag toggles between legal and pseudo-legal move
    /// generation for `false` and `true` respectively.
    ///
    /// The `QUIET` and `NOISY` flags toggles the generation of reversible and
    /// irreversible moves respectively.
    fn generate_moves_into<
        const ALLOW_ILLEGAL: bool,
        const QUIET: bool,
        const NOISY: bool,
        T: MoveStore<Self::Move>,
    >(
        &self,
        movelist: &mut T,
    );
    /// `generate_moves` is similar to `generate_moves_into`, except that
    /// instead of taking some storage as input it stores into a [MoveList].
    #[must_use]
    fn generate_moves<
        const ALLOW_ILLEGAL: bool,
        const QUIET: bool,
        const NOISY: bool,
    >(
        &self,
    ) -> MoveList<Self::Move> {
        let mut movelist: MoveList<Self::Move> = Default::default();
        self.generate_moves_into::<ALLOW_ILLEGAL, QUIET, NOISY, _>(
            &mut movelist,
        );
        movelist
    }
    /// `count_moves` is similar to `generate_moves`, except instead of
    /// returning a list of the available moves, it returns the number of
    /// available moves.
    ///
    /// By default this is simply `generate_moves().len()`, but implementations
    /// may take advantage of various optimizations counting as opposed to
    /// storing the moves allows to provide a more efficient version.
    #[must_use]
    fn count_moves<const QUIET: bool, const NOISY: bool>(&self) -> usize {
        self.generate_moves::<false, QUIET, NOISY>().len()
    }
}
