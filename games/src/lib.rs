use std::fmt::Display;
use std::ops::{Index, Not};
use std::str::FromStr;

use arrayvec::ArrayVec;

pub mod ataxx;

pub trait Position:
    FromStr
    + Display
    + Index<<Self::ColoredPiece as ColoredPiece>::Piece>
    + Index<<Self::ColoredPiece as ColoredPiece>::Color>
    + Index<Self::ColoredPiece>
where
    Self::Square: Square,
    Self::BitBoard: BitBoard,
    Self::ColoredPiece: ColoredPiece,
    Self::Move: Move,
{
    type Square;
    type BitBoard;

    type ColoredPiece;

    type Move;

    fn put(&mut self, sq: Self::Square, piece: <Self::ColoredPiece as ColoredPiece>::Piece);
    fn at(&self, sq: Self::Square) -> Option<<Self::ColoredPiece as ColoredPiece>::Piece>;

    fn is_game_over(&self) -> bool {
        self.winner().is_some()
    }
    fn winner(&self) -> Option<<Self::ColoredPiece as ColoredPiece>::Color>;
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

pub trait TypeEnum<B>: Copy + Eq + FromStr + Display + From<B> + Into<B> {
    const N: usize;
    fn unsafe_from<T: Into<B>>(number: T) -> Self;
}

pub trait Move: Default + FromStr + Display {}
pub trait Color: TypeEnum<u8> + Not {}
pub trait ColoredPiece: TypeEnum<u8>
where
    Self::Piece: TypeEnum<u8>,
    Self::Color: Color,
{
    type Piece;
    type Color;

    fn new(piece: Self::Piece, color: Self::Color) -> Self {
        Self::unsafe_from(color.into() * Self::Piece::N as u8 + piece.into())
    }

    fn piece(&self) -> Self::Piece;
    fn color(&self) -> Self::Color;
}
pub trait Square: TypeEnum<u8>
where
    Self::File: TypeEnum<u8>,
    Self::Rank: TypeEnum<u8>,
{
    type File;
    type Rank;

    fn new(file: Self::File, rank: Self::Rank) -> Self {
        Self::unsafe_from(rank.into() * Self::File::N as u8 + file.into())
    }

    fn file(self) -> Self::File {
        Self::File::unsafe_from(self.into() % Self::File::N as u8)
    }

    fn rank(self) -> Self::Rank {
        Self::Rank::unsafe_from(self.into() / Self::File::N as u8)
    }

    fn north(self) -> Self {
        Self::unsafe_from(self.into() + Self::File::N as u8)
    }

    fn south(self) -> Self {
        Self::unsafe_from(self.into() - Self::File::N as u8)
    }

    fn east(self) -> Self {
        Self::unsafe_from(self.into() + 1)
    }

    fn west(self) -> Self {
        Self::unsafe_from(self.into() - 1)
    }
}
pub trait BitBoard {}

/// MoveStore is a trait implemented by types which are able to store moves
/// inside themselves and are thus usable in move-generation methods in
/// [Position] like [`Position::generate_moves_into<T>`].
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
