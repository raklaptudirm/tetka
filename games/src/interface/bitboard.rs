use std::{
    marker::PhantomData,
    ops::{BitAnd, BitOr, BitXor, Shl, Shr},
};

use num_traits::int::PrimInt;

use super::{
    CartesianFile, CartesianRank, CartesianSquare, RepresentableType, SetType,
    SquareType,
};

/// BitBoardType is a generalized interface implemented by BitBoards of
/// arbitrary size. This allows programs to handle BitBoards of any size with
/// generic functions using this common interface.
pub trait BitBoardType: SetType<Self::Base, Self::Square, u8>
where
    Self::Base: PrimInt,
    Self::Square: SquareType,
{
    /// The backing [`PrimInt`] type for the BitBoard.
    type Base;
    /// The type of the Squares in the BitBoard.
    type Square;

    /// The BitBoard containing Squares in the first File.
    const FIRST_FILE: Self;
    /// The BitBoard containing Squares in the first Rank.
    const FIRST_RANK: Self;

    /// north returns a new Self with all the squares shifted to the north.
    #[must_use]
    fn north(self) -> Self {
        (self << <Self::Square as SquareType>::File::N) & Self::UNIVERSE
    }

    /// south returns a new Self with all the squares shifted to the south.
    #[must_use]
    fn south(self) -> Self {
        self >> <Self::Square as SquareType>::File::N
    }

    /// east returns a new Self with all the squares shifted to the east.
    #[must_use]
    fn east(self) -> Self {
        (self << 1) & (Self::UNIVERSE ^ Self::FIRST_FILE)
    }

    /// west returns a new Self with all the squares shifted to the west.
    #[must_use]
    fn west(self) -> Self {
        (self >> 1)
            & (Self::UNIVERSE
                ^ (Self::FIRST_FILE
                    << (<Self::Square as SquareType>::File::N - 1)))
    }

    /// Returns a BitBoard containing all the squares from the given `File`.
    #[must_use]
    fn file(file: <Self::Square as SquareType>::File) -> Self {
        Self::FIRST_FILE << file.into() as usize
    }

    /// Returns a BitBoard containing all the squares from the given `Rank`.
    #[must_use]
    fn rank(rank: <Self::Square as SquareType>::Rank) -> Self {
        Self::FIRST_RANK
            << (<Self::Square as SquareType>::File::N * rank.into() as usize)
    }
}

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    //num_derive::FromPrimitive,
    //derive_more::BitOr,
    //derive_more::BitAnd,
    //derive_more::BitXor,
    //derive_more::Shl,
    //derive_more::Shr,
    //derive_more::BitAndAssign,
    //derive_more::BitOrAssign,
    //derive_more::BitXorAssign,
    //derive_more::ShlAssign,
    //derive_more::ShrAssign,
    //derive_more::From,
    //derive_more::Into,
)]
pub struct CartesianSquareSet<B: PrimInt, const F: u8, const R: u8>(
    u64,
    PhantomData<B>,
);

impl<B: PrimInt, const F: u8, const R: u8> CartesianSquareSet<B, F, R> {
    /// The BitBoard containing Squares in the first File.
    const FIRST_FILE: Self = Self(0, PhantomData);
    /// The BitBoard containing Squares in the first Rank.
    const FIRST_RANK: Self = Self(0, PhantomData);

    /// north returns a new Self with all the squares shifted to the north.
    #[must_use]
    fn north(self) -> Self {
        (self << F) & Self::UNIVERSE
    }

    /// south returns a new Self with all the squares shifted to the south.
    #[must_use]
    fn south(self) -> Self {
        self >> F
    }

    /// east returns a new Self with all the squares shifted to the east.
    #[must_use]
    fn east(self) -> Self {
        (self << 1u8) & (Self::UNIVERSE ^ Self::FIRST_FILE)
    }

    /// west returns a new Self with all the squares shifted to the west.
    #[must_use]
    fn west(self) -> Self {
        (self >> 1u8) & (Self::UNIVERSE ^ (Self::FIRST_FILE << (F - 1)))
    }

    /// Returns a BitBoard containing all the squares from the given `File`.
    #[must_use]
    fn file(file: CartesianFile<F>) -> Self {
        Self::FIRST_FILE << u8::from(file) as usize
    }

    /// Returns a BitBoard containing all the squares from the given `Rank`.
    #[must_use]
    fn rank(rank: CartesianRank<R>) -> Self {
        Self::FIRST_RANK << (F as usize * u8::from(rank) as usize)
    }
}

// REMOVE //

impl<B: PrimInt, const F: u8, const R: u8> BitOr
    for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, self.1)
    }
}

impl<B: PrimInt, const F: u8, const R: u8> BitAnd
    for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0, self.1)
    }
}
impl<B: PrimInt, const F: u8, const R: u8> BitXor
    for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0, self.1)
    }
}

impl<B: PrimInt, const F: u8, const R: u8> Shl<u8>
    for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        Self(self.0 << rhs, self.1)
    }
}

impl<B: PrimInt, const F: u8, const R: u8> Shr<u8>
    for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        Self(self.0 >> rhs, self.1)
    }
}

impl<B: PrimInt, const F: u8, const R: u8> Shl<usize>
    for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self(self.0 << rhs, self.1)
    }
}

impl<B: PrimInt, const F: u8, const R: u8> Shr<usize>
    for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self(self.0 >> rhs, self.1)
    }
}
// REMOVE //

impl<B: PrimInt, const F: u8, const R: u8>
    SetType<u64, CartesianSquare<B, F, R>, B> for CartesianSquareSet<B, F, R>
{
    const EMPTY: Self = Self(0, PhantomData);
    const UNIVERSE: Self = Self(0, PhantomData);
}

impl<B: PrimInt, const F: u8, const R: u8> Iterator
    for CartesianSquareSet<B, F, R>
{
    type Item = CartesianSquare<B, F, R>;

    /// next pops the next Square from the BitBoard and returns it.
    fn next(&mut self) -> Option<Self::Item> {
        use crate::interface::SetType;
        let lsb = if self.is_empty() {
            None
        } else {
            let sq = <Self as Into<u64>>::into(*self).trailing_zeros() as usize;
            Some(unsafe { CartesianSquare::<B, F, R>::unsafe_from(sq) })
        };

        if !self.is_empty() {
            let copy = u64::from(*self);
            self.0 = copy & (copy - u64::from(1u8));
        }

        lsb
    }
}

impl<B: PrimInt, const F: u8, const R: u8> From<CartesianSquareSet<B, F, R>>
    for u64
{
    fn from(value: CartesianSquareSet<B, F, R>) -> Self {
        value.0
    }
}

// a -> {a}
impl<B: PrimInt, const F: u8, const R: u8> From<CartesianSquare<B, F, R>>
    for CartesianSquareSet<B, F, R>
{
    #[must_use]
    fn from(square: CartesianSquare<B, F, R>) -> Self {
        Self(
            u64::from(1u8) << square.into().to_u16().unwrap(),
            PhantomData,
        )
    }
}

// The set complement operator (!).
impl<B: PrimInt, const F: u8, const R: u8> std::ops::Not
    for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    /// Returns the complementary BitBoard of `self`.
    #[must_use]
    fn not(self) -> Self::Output {
        use SetType;
        // ! will set the unused bits so remove them with an &.
        Self(!self.0, PhantomData) & Self::UNIVERSE
    }
}

// The set difference operator (-).
#[allow(clippy::suspicious_arithmetic_impl)]
impl<B: PrimInt, const F: u8, const R: u8> std::ops::Sub
    for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    /// Returns the difference of `self` and `rhs` as a new BitBoard.
    #[must_use]
    fn sub(self, rhs: Self) -> Self::Output {
        self & !rhs
    }
}

// Assignment version of the set difference operator.
#[allow(clippy::suspicious_arithmetic_impl)]
impl<B: PrimInt, const F: u8, const R: u8> std::ops::SubAssign
    for CartesianSquareSet<B, F, R>
{
    /// Returns the difference of `self` and `rhs` as a new BitBoard.
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self & !rhs
    }
}

// A | {a}
#[allow(clippy::suspicious_arithmetic_impl)]
impl<B: PrimInt, const F: u8, const R: u8>
    std::ops::BitOr<CartesianSquare<B, F, R>> for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    /// Returns the union of `self` and `rhs` as a new BitBoard.
    #[must_use]
    fn bitor(self, rhs: CartesianSquare<B, F, R>) -> Self::Output {
        self | Self::from(rhs)
    }
}

// A - {a}
impl<B: PrimInt, const F: u8, const R: u8>
    std::ops::Sub<CartesianSquare<B, F, R>> for CartesianSquareSet<B, F, R>
{
    type Output = Self;

    /// Returns the BitBoard obtained on removing `rhs` from `self`.
    #[must_use]
    fn sub(self, rhs: CartesianSquare<B, F, R>) -> Self::Output {
        self & !Self::from(rhs)
    }
}
