use std::fmt::Display;
use std::str::FromStr;

use arrayvec::ArrayVec;

/// The Move trait should be implemented the move representation of a game.
pub trait MoveType: FromStr + Display + From<u16> + Into<u16> + Copy {
    /// NULL represents the null or the 'do nothing' move.
    const NULL: Self;
    /// MAX_IN_GAME is a suitably high maximum for the number of move in a game.
    const MAX_IN_GAME: usize;
    /// MAX_IN_POSITION is a suitably high maximum for the number of move in a
    /// single, possibly unreachable position.
    const MAX_IN_POSITION: usize;
}

/// MoveStore is a trait implemented by types which are able to store moves
/// inside themselves and are thus usable in move-generation methods in
/// [Position](super::PositionType) like
/// [`generate_moves_into<T>`](super::PositionType::generate_moves_into<T>).
pub trait MoveStore<M>: Default {
    /// Appends a move to the back of the [MoveStore].
    fn push(&mut self, m: M);

    /// Returns the number of moves in the [MoveStore], also referred to as its
    /// ‘length’.
    #[must_use]
    fn len(&self) -> usize;

    /// Returns `true` if the [MoveStore] contains no moves.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// MoveList is a basic implementation of [`MoveStore`] that is used to allow users
/// to utilize move-generation methods without having to implement a [MoveStore] by
/// themselves. It also has utility methods other than the [`MoveStore`] trait.
///
/// MoveList is allocated on the stack and very fast for use. However, due to
/// current limitations in the Rust type system, the current max capacity is
/// capped at 256, which can be problematic for games which can have more moves
/// in a position and might require a custom type.
pub type MoveList<M> = ArrayVec<M, 256>;

// MoveStore implementation for MoveList.
impl<M> MoveStore<M> for MoveList<M> {
    fn push(&mut self, m: M) {
        self.push(m);
    }

    #[must_use]
    fn len(&self) -> usize {
        self.len()
    }

    #[must_use]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
