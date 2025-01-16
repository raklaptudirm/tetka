use std::{fmt::Display, str::FromStr};

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

#[derive(Clone, Copy, derive_more::Into)]
pub struct CartesianSquare<const F: u8, const R: u8>(u16);

impl<const F: u8, const R: u8> CartesianSquare<F, R> {
    /// Creates a new Square from the given File and Rank.
    #[must_use]
    pub fn new(file: CartesianFile<F>, rank: CartesianRank<R>) -> Self {
        Self((u8::from(rank) * F + u8::from(file)) as u16)
    }

    /// Returns the File of self.
    #[must_use]
    pub fn file(self) -> CartesianFile<F> {
        CartesianFile::<F>((u16::from(self) % F as u16) as u8)
    }

    /// Returns the Rank of self.
    #[must_use]
    pub fn rank(self) -> CartesianRank<R> {
        CartesianRank::<R>((u16::from(self) / F as u16) as u8)
    }

    #[must_use]
    pub fn north(self) -> Option<Self> {
        if self.rank().last() {
            None
        } else {
            Some(Self(u16::from(self) + F as u16))
        }
    }

    /// Returns the square to the south of self. If there is no Square to the
    /// south of self, it returns None.
    #[must_use]
    pub fn south(self) -> Option<Self> {
        if self.rank().first() {
            None
        } else {
            Some(Self(u16::from(self) - F as u16))
        }
    }

    /// Returns the square to the east of self. If there is no Square to the
    /// east of self, it returns None.
    #[must_use]
    pub fn east(self) -> Option<Self> {
        if self.file().last() {
            None
        } else {
            Some(Self(u16::from(self) + 1))
        }
    }

    /// Returns the square to the west of self. If there is no Square to the
    /// west of self, it returns None.
    #[must_use]
    pub fn west(self) -> Option<Self> {
        if self.file().first() {
            None
        } else {
            Some(Self(u16::from(self) - 1))
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

impl<const F: u8, const R: u8> Display for CartesianSquare<F, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

#[derive(Clone, Copy, derive_more::Into)]
pub struct CartesianFile<const N: u8>(u8);

impl<const N: u8> CartesianFile<N> {
    pub fn first(self) -> bool {
        u8::from(self) == 0
    }

    pub fn last(self) -> bool {
        u8::from(self) + 1 == N
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

#[derive(Clone, Copy, derive_more::Into)]
pub struct CartesianRank<const N: u8>(u8);

impl<const N: u8> CartesianRank<N> {
    pub fn first(self) -> bool {
        u8::from(self) == 0
    }

    pub fn last(self) -> bool {
        u8::from(self) + 1 == N
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
