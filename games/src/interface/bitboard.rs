use std::{
    marker::PhantomData,
    ops::{BitAnd, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, Shr},
};

use num_traits::int::PrimInt;

use super::{
    CartesianFile, CartesianRank, CartesianSquare, RepresentableType, SetType,
};

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
    pub fn north(self) -> Self {
        (self << F) & Self::UNIVERSE
    }

    /// south returns a new Self with all the squares shifted to the south.
    #[must_use]
    pub fn south(self) -> Self {
        self >> F
    }

    /// east returns a new Self with all the squares shifted to the east.
    #[must_use]
    pub fn east(self) -> Self {
        (self << 1u8) & (Self::UNIVERSE ^ Self::FIRST_FILE)
    }

    /// west returns a new Self with all the squares shifted to the west.
    #[must_use]
    pub fn west(self) -> Self {
        (self >> 1u8) & (Self::UNIVERSE ^ (Self::FIRST_FILE << (F - 1)))
    }

    /// Returns a BitBoard containing all the squares from the given `File`.
    #[must_use]
    pub fn file(file: CartesianFile<F>) -> Self {
        Self::FIRST_FILE << u8::from(file) as usize
    }

    /// Returns a BitBoard containing all the squares from the given `Rank`.
    #[must_use]
    pub fn rank(rank: CartesianRank<R>) -> Self {
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

impl<B: PrimInt, const F: u8, const R: u8> BitOrAssign
    for CartesianSquareSet<B, F, R>
{
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl<B: PrimInt, const F: u8, const R: u8> BitXorAssign
    for CartesianSquareSet<B, F, R>
{
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
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

impl<B: PrimInt, const F: u8, const R: u8> From<u64>
    for CartesianSquareSet<B, F, R>
{
    fn from(value: u64) -> Self {
        Self(value, PhantomData)
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
