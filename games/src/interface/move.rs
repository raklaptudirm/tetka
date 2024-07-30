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
    /// push adds the given Move to the MoveStore.
    fn push(&mut self, m: M);

    /// len returns the number of [Moves](MoveType) stored in the MoveStore.
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
