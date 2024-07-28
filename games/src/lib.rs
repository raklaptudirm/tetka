use std::fmt::Display;
use std::ops::{Index, Not};
use std::str::FromStr;

use arrayvec::ArrayVec;
use num_traits::ToPrimitive;

pub mod ataxx;

pub trait Position:
    FromStr + Display + Index<Self::Piece> + Index<Self::Color> + Index<Self::ColoredPiece>
where
    Self::Square: Square,
    Self::BitBoard: BitBoard,
    Self::Piece: TypeEnum,
    Self::Color: Color,
    Self::ColoredPiece: TypeEnum,
    Self::Move: Move,
{
    type Square;
    type BitBoard;

    type Piece;
    type Color;
    type ColoredPiece;

    type Move;

    fn put(&mut self, sq: Self::Square, piece: Self::Piece);
    fn at(&self, sq: Self::Square) -> Option<Self::Piece>;

    fn is_game_over(&self) -> bool {
        self.winner().is_some()
    }
    fn winner(&self) -> Option<Self::Color>;
    fn after_move<const UPDATE_HASH: bool>(mov: Self::Move) -> Self;

    fn generate_moves_into<T: MoveStore<Self::Move>>(&self, movelist: &mut T);
    fn generate_moves(&self) -> MoveList<Self::Move> {
        let mut movelist: MoveList<Self::Move> = Default::default();
        self.generate_moves_into(&mut movelist);
        movelist
    }
    fn count_moves(&self) -> usize {
        self.generate_moves().len()
    }
}

pub trait TypeEnum: Copy + Eq + ToPrimitive + FromStr + Display {
    const N: usize;
    fn unsafe_from<T: num_traits::ToPrimitive>(number: T) -> Self;
}

pub trait Move: Default + FromStr + Display {}
pub trait Color: TypeEnum + Not {}
pub trait Square: TypeEnum
where
    Self::File: TypeEnum,
    Self::Rank: TypeEnum,
{
    type File;
    type Rank;

    fn new(file: Self::File, rank: Self::Rank) -> Self {
        Self::unsafe_from(
            unsafe { rank.to_usize().unwrap_unchecked() } * Self::File::N
                + unsafe { file.to_usize().unwrap_unchecked() },
        )
    }

    fn file(self) -> Self::File {
        Self::File::unsafe_from(unsafe { self.to_usize().unwrap_unchecked() } % Self::File::N)
    }

    fn rank(self) -> Self::Rank {
        Self::Rank::unsafe_from(unsafe { self.to_usize().unwrap_unchecked() } / Self::File::N)
    }

    fn north(self) -> Self {
        Self::unsafe_from(unsafe { self.to_usize().unwrap_unchecked() } + Self::File::N)
    }

    fn south(self) -> Self {
        Self::unsafe_from(unsafe { self.to_usize().unwrap_unchecked() } - Self::File::N)
    }

    fn east(self) -> Self {
        Self::unsafe_from(unsafe { self.to_usize().unwrap_unchecked() } + 1)
    }

    fn west(self) -> Self {
        Self::unsafe_from(unsafe { self.to_usize().unwrap_unchecked() } - 1)
    }
}
pub trait BitBoard {}

/// MoveStore is a trait implemented by types which are able to store moves inside
/// themselves and are thus usable in move-generation methods in
/// [Position](super::Position) like
/// [`Position::generate_moves_into<T>`](super::Position::generate_moves_into<T>).
pub trait MoveStore<M>: Default {
    /// push adds the given Move to the MoveStore.
    fn push(&mut self, m: M);

    /// len returns the number of [Move]s stored in the MoveStore.
    fn len(&self) -> usize;

    /// is_empty checks if no [Move]s are stored in the MoveStore.
    fn is_empty(&self) -> bool;
}

/// MoveList is a basic implementation of [`MoveStore`] that is used to allow users
/// to utilize move-generation methods without having to implement a [MoveStore] by
/// themselves. It also has utility methods other than the [`MoveStore`] trait.
pub type MoveList<M> = ArrayVec<M, 256>;

impl<M: Default> MoveStore<M> for MoveList<M> {
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
