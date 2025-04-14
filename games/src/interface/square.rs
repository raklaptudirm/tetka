use std::{fmt::Display, iter::FusedIterator, str::FromStr};

use num_traits::{FromPrimitive, PrimInt};
use strum::IntoEnumIterator;

use super::{RepresentableType, TypeParseError};

/// SquareType is the trait implemented by the type for Squares in a game's
/// board representation. It provides a generic interface along with useful
/// predefined methods that a Square type would require.
pub trait SquareType: RepresentableType<u8>
where
    Self::File: RepresentableType<u8>,
    Self::Rank: RepresentableType<u8>,
{
    /// The type for the File of the Square.
    type File;
    /// The type for the Rank of the Square.
    type Rank;

    /// Creates a new Square from the given File and Rank.
    #[must_use]
    fn new(file: Self::File, rank: Self::Rank) -> Self {
        unsafe {
            Self::unsafe_from(rank.into() * Self::File::N as u8 + file.into())
        }
    }

    /// Returns the File of self.
    #[must_use]
    fn file(self) -> Self::File {
        unsafe { Self::File::unsafe_from(self.into() % Self::File::N as u8) }
    }

    /// Returns the Rank of self.
    #[must_use]
    fn rank(self) -> Self::Rank {
        unsafe { Self::Rank::unsafe_from(self.into() / Self::File::N as u8) }
    }

    /// Returns the square to the north of self. If there is no Square to the
    /// north of self, it returns None.
    #[must_use]
    fn north(self) -> Option<Self> {
        if self.rank().into() as usize == Self::Rank::N - 1 {
            None
        } else {
            Some(unsafe {
                Self::unsafe_from(self.into() + Self::File::N as u8)
            })
        }
    }

    /// Returns the square to the south of self. If there is no Square to the
    /// south of self, it returns None.
    #[must_use]
    fn south(self) -> Option<Self> {
        if self.rank().into() as usize == 0 {
            None
        } else {
            Some(unsafe {
                Self::unsafe_from(self.into() - Self::File::N as u8)
            })
        }
    }

    /// Returns the square to the east of self. If there is no Square to the
    /// east of self, it returns None.
    #[must_use]
    fn east(self) -> Option<Self> {
        if self.file().into() as usize == Self::File::N - 1 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() + 1) })
        }
    }

    /// Returns the square to the west of self. If there is no Square to the
    /// west of self, it returns None.
    #[must_use]
    fn west(self) -> Option<Self> {
        if self.file().into() as usize == 0 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() - 1) })
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, derive_more::Into)]
pub struct CartesianSquare<B: PrimInt, const F: u8, const R: u8>(B);

impl<B: PrimInt, const F: u8, const R: u8> CartesianSquare<B, F, R> {
    /// Creates a new Square from the given File and Rank.
    #[must_use]
    pub fn new(file: CartesianFile<F>, rank: CartesianRank<R>) -> Self {
        Self(
            ((u8::from(rank) * F + u8::from(file)) as u16)
                .into()
                .unwrap(),
        )
    }

    /// Returns the File of self.
    #[must_use]
    pub fn file(self) -> CartesianFile<F> {
        CartesianFile::<F>(u8::from(self.into() % B::from(F).unwrap()))
    }

    /// Returns the Rank of self.
    #[must_use]
    pub fn rank(self) -> CartesianRank<R> {
        CartesianRank::<R>(u8::from(self.into() / B::from(F).unwrap()))
    }

    #[must_use]
    pub fn north(self) -> Option<Self> {
        if self.rank().is_last() {
            None
        } else {
            Some(Self(self.into() + B::from(F).unwrap()))
        }
    }

    /// Returns the square to the south of self. If there is no Square to the
    /// south of self, it returns None.
    #[must_use]
    pub fn south(self) -> Option<Self> {
        if self.rank().is_first() {
            None
        } else {
            Some(Self(self.into() - B::from(F).unwrap()))
        }
    }

    /// Returns the square to the east of self. If there is no Square to the
    /// east of self, it returns None.
    #[must_use]
    pub fn east(self) -> Option<Self> {
        if self.file().is_last() {
            None
        } else {
            Some(Self(self.into() + B::one()))
        }
    }

    /// Returns the square to the west of self. If there is no Square to the
    /// west of self, it returns None.
    #[must_use]
    pub fn west(self) -> Option<Self> {
        if self.file().is_first() {
            None
        } else {
            Some(Self(self.into() - B::one()))
        }
    }
}

impl<B: PrimInt, const F: u8, const R: u8> FromStr
    for CartesianSquare<B, F, R>
{
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(
            CartesianFile::<F>::from_str(&s[..1])?,
            CartesianRank::<R>::from_str(&s[1..])?,
        ))
    }
}

impl<B: PrimInt, const F: u8, const R: u8> Display
    for CartesianSquare<B, F, R>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl<B: PrimInt, const F: u8, const R: u8> From<B>
    for CartesianSquare<B, F, R>
{
    fn from(value: B) -> Self {
        Self(value)
    }
}

impl<B: PrimInt, const F: u8, const R: u8> RepresentableType<B>
    for CartesianSquare<B, F, R>
{
    const N: usize = F as usize * R as usize;
    unsafe fn unsafe_from<T: PrimInt>(number: T) -> Self {
        Self(B::from(number).unwrap_unchecked())
    }
}

impl<B: PrimInt, const F: u8, const R: u8> IntoEnumIterator
    for CartesianSquare<B, F, R>
{
    type Iterator = Self;

    fn iter() -> Self::Iterator {
        Self(B::zero())
    }
}

impl<B: PrimInt, const F: u8, const R: u8> Iterator
    for CartesianSquare<B, F, R>
{
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = (*self).into();
        if idx < B::from(F).unwrap() * B::from(R).unwrap() {
            self.0 = idx + B::one();
            Some(Self(idx))
        } else {
            None
        }
    }
}

impl<B: PrimInt, const F: u8, const R: u8> DoubleEndedIterator
    for CartesianSquare<B, F, R>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let idx = (*self).into();
        self.0 = idx - B::one();
        if idx < B::from(F).unwrap() * B::from(R).unwrap() {
            Some(Self(idx))
        } else {
            None
        }
    }
}

impl<B: PrimInt, const F: u8, const R: u8> ExactSizeIterator
    for CartesianSquare<B, F, R>
{
}
impl<B: PrimInt, const F: u8, const R: u8> FusedIterator
    for CartesianSquare<B, F, R>
{
}

#[derive(Clone, Copy, PartialEq, Eq, derive_more::Into, derive_more::From)]
pub struct CartesianFile<const N: u8>(u8);

impl<const N: u8> CartesianFile<N> {
    pub fn first() -> Self {
        Self(0)
    }

    pub fn last() -> Self {
        Self(N - 1)
    }

    pub fn is_first(self) -> bool {
        self == Self::first()
    }

    pub fn is_last(self) -> bool {
        self == Self::last()
    }
}

impl<const N: u8> FromStr for CartesianFile<N> {
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(TypeParseError::StrError(
                stringify!(CartesianFile<N>).to_string(),
            ));
        }

        let idx =
            unsafe { s.chars().next().unwrap_unchecked() } as u8 - 'a' as u8;
        if idx < N {
            Ok(Self(idx))
        } else {
            Err(TypeParseError::RangeError(
                stringify!(CartesianFile<N>).to_string(),
            ))
        }
    }
}

impl<const N: u8> Display for CartesianFile<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ('a' as u8 + u8::from(*self)) as char)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, derive_more::Into, derive_more::From)]
pub struct CartesianRank<const N: u8>(u8);

impl<const N: u8> CartesianRank<N> {
    pub fn first() -> Self {
        Self(0)
    }

    pub fn last() -> Self {
        Self(N - 1)
    }

    pub fn is_first(self) -> bool {
        self == Self::first()
    }

    pub fn is_last(self) -> bool {
        self == Self::last()
    }
}

impl<const N: u8> FromStr for CartesianRank<N> {
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse().map_err(|_| {
            TypeParseError::StrError(stringify!(CartesianRank<N>).to_string())
        })?)
    }
}

impl<const N: u8> Display for CartesianRank<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u8::from(*self) + 1)
    }
}
