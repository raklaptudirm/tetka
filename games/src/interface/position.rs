use std::fmt::Display;
use std::str::FromStr;

use super::{
    Color, ColoredPieceType, Hash, MoveList, MoveStore, MoveType, SquareType,
};

/// A generalized interface for board representations of a wide range of games.
///
/// It is designed to create game-agnostic software. Tetka provides the logic
/// for many popular games out of the box, but custom games can easily be
/// implemented by the library user.
pub trait PositionType: FromStr + Display
where
    Self::ColoredPiece: ColoredPieceType,
    Self::Move: MoveType,
{
    /// Type for the squares in the board representation.
    type Square: SquareType;

    /// Type for the pieces (with color) used by this board representation.
    type ColoredPiece;

    /// Type for one move in this board representation.
    type Move;

    /// FEN string for the standard starting position of the game.
    ///
    /// If the game doesn't have a standard starting a position, any legal
    /// starting position or a de-facto standard may be used.
    const STARTPOS: &str;

    /// Adds the given Piece to the given Square. If the target Square is
    /// non-empty, the behavior is undefined.
    fn insert(&mut self, sq: Self::Square, piece: Self::ColoredPiece);
    /// Removes any Piece on the given Square, and returns the removed Piece.
    /// For games where there may be multiple pieces on a single Square,
    /// it removes only the 'topmost' Piece.
    fn remove(&mut self, sq: Self::Square) -> Option<Self::ColoredPiece>;
    /// Returns the Piece present at the given Square.
    #[must_use]
    fn at(&self, sq: Self::Square) -> Option<Self::ColoredPiece>;

    /// Returns the current side to move.
    #[must_use]
    fn side_to_move(&self) -> Color<Self>;
    /// Returns the value of half-move draw clock.
    #[must_use]
    fn half_move_clock(&self) -> usize;
    /// Returns the number of plys played till now.
    #[must_use]
    fn ply_count(&self) -> usize;
    /// Returns a semi-unique checksum of the current Position.
    #[must_use]
    fn hash(&self) -> Hash;

    /// Returns the side which has won in the current position, if any.
    #[must_use]
    fn winner(&self) -> Option<Option<Color<Self>>>;
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
    ) -> impl MoveStore<Self::Move> {
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
