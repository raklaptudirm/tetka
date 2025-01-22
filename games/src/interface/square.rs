use std::{
    fmt::Display,
    iter::FusedIterator,
    ops::{Index, IndexMut},
    str::FromStr,
};

use num_traits::PrimInt;
use strum::IntoEnumIterator;

use super::{RepresentableType, TypeParseError};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct CartesianSquare<const F: u8, const R: u8>(u8);

impl<const F: u8, const R: u8> CartesianSquare<F, R> {
    pub const N: usize = F as usize * R as usize;

    /// Creates a new Square from the given File and Rank.
    #[must_use]
    pub fn new(file: CartesianFile<F>, rank: CartesianRank<R>) -> Self {
        Self(u8::from(rank) * F + u8::from(file))
    }

    /// Returns the File of self.
    #[must_use]
    pub fn file(self) -> CartesianFile<F> {
        CartesianFile::<F>(u8::from(self) % F)
    }

    /// Returns the Rank of self.
    #[must_use]
    pub fn rank(self) -> CartesianRank<R> {
        CartesianRank::<R>(u8::from(self) / F)
    }

    #[must_use]
    pub fn north(self) -> Option<Self> {
        if self.rank().is_last() {
            None
        } else {
            Some(Self(u8::from(self) + F))
        }
    }

    /// Returns the square to the south of self. If there is no Square to the
    /// south of self, it returns None.
    #[must_use]
    pub fn south(self) -> Option<Self> {
        if self.rank().is_first() {
            None
        } else {
            Some(Self(u8::from(self) - F))
        }
    }

    /// Returns the square to the east of self. If there is no Square to the
    /// east of self, it returns None.
    #[must_use]
    pub fn east(self) -> Option<Self> {
        if self.file().is_last() {
            None
        } else {
            Some(Self(u8::from(self) + 1))
        }
    }

    /// Returns the square to the west of self. If there is no Square to the
    /// west of self, it returns None.
    #[must_use]
    pub fn west(self) -> Option<Self> {
        if self.file().is_first() {
            None
        } else {
            Some(Self(u8::from(self) - 1))
        }
    }
}

impl<const F: u8, const R: u8> FromStr for CartesianSquare<F, R> {
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(
            CartesianFile::<F>::from_str(&s[..1])?,
            CartesianRank::<R>::from_str(&s[1..])?,
        ))
    }
}

impl<const F: u8, const R: u8> From<CartesianSquare<F, R>> for u8 {
    fn from(value: CartesianSquare<F, R>) -> Self {
        value.0
    }
}

impl<const F: u8, const R: u8> Display for CartesianSquare<F, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl<const F: u8, const R: u8> From<u8> for CartesianSquare<F, R> {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl<const F: u8, const R: u8> RepresentableType<u8> for CartesianSquare<F, R> {
    const N: usize = F as usize * R as usize;
    unsafe fn unsafe_from<T: PrimInt>(number: T) -> Self {
        Self(number.to_u8().unwrap_unchecked())
    }
}

impl<A, const F: u8, const R: u8, const N: usize> Index<CartesianSquare<F, R>>
    for [A; N]
{
    type Output = A;

    fn index(&self, index: CartesianSquare<F, R>) -> &Self::Output {
        &self[u8::from(index) as usize]
    }
}

impl<A, const F: u8, const R: u8, const N: usize>
    IndexMut<CartesianSquare<F, R>> for [A; N]
{
    fn index_mut(&mut self, index: CartesianSquare<F, R>) -> &mut Self::Output {
        &mut self[u8::from(index) as usize]
    }
}

impl<const F: u8, const R: u8> IntoEnumIterator for CartesianSquare<F, R> {
    type Iterator = Self;

    fn iter() -> Self::Iterator {
        Self(0)
    }
}

impl<const F: u8, const R: u8> Iterator for CartesianSquare<F, R> {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = (*self).into();
        if idx < F * R {
            self.0 = idx + 1;
            Some(Self(idx))
        } else {
            None
        }
    }
}

impl<const F: u8, const R: u8> DoubleEndedIterator for CartesianSquare<F, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let idx = (*self).into();
        self.0 = idx - 1;
        if idx < F * R {
            Some(Self(idx))
        } else {
            None
        }
    }
}

impl<const F: u8, const R: u8> FusedIterator for CartesianSquare<F, R> {}
impl<const F: u8, const R: u8> ExactSizeIterator for CartesianSquare<F, R> {}

#[derive(Clone, Copy, PartialEq, Eq, derive_more::Into, derive_more::From)]
pub struct CartesianFile<const N: u8>(u8);

impl<const N: u8> CartesianFile<N> {
    pub const N: usize = N as usize;

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

impl<A, const C: u8, const N: usize> Index<CartesianFile<C>> for [A; N] {
    type Output = A;

    fn index(&self, index: CartesianFile<C>) -> &Self::Output {
        &self[u8::from(index) as usize]
    }
}

impl<A, const C: u8, const N: usize> IndexMut<CartesianFile<C>> for [A; N] {
    fn index_mut(&mut self, index: CartesianFile<C>) -> &mut Self::Output {
        &mut self[u8::from(index) as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, derive_more::Into, derive_more::From)]
pub struct CartesianRank<const N: u8>(u8);

impl<const N: u8> CartesianRank<N> {
    pub const N: usize = N as usize;
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

impl<A, const C: u8, const N: usize> Index<CartesianRank<C>> for [A; N] {
    type Output = A;

    fn index(&self, index: CartesianRank<C>) -> &Self::Output {
        &self[u8::from(index) as usize]
    }
}

impl<A, const C: u8, const N: usize> IndexMut<CartesianRank<C>> for [A; N] {
    fn index_mut(&mut self, index: CartesianRank<C>) -> &mut Self::Output {
        &mut self[u8::from(index) as usize]
    }
}
