// Copyright © 2024 Rak Laptudirm <rak@laptudirm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;
use std::str::FromStr;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum_macros::EnumIter;
use thiserror::Error;

use crate::type_macros;

/// Square represents all the squares present on an Ataxx Board.
/// The index of each Square is equal to `rank-index * 8 + file-index`.
#[derive(Copy, Clone, PartialEq, PartialOrd, FromPrimitive, EnumIter)]
#[rustfmt::skip]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1,
    A2, B2, C2, D2, E2, F2, G2,
    A3, B3, C3, D3, E3, F3, G3,
    A4, B4, C4, D4, E4, F4, G4,
    A5, B5, C5, D5, E5, F5, G5,
    A6, B6, C6, D6, E6, F6, G6,
    A7, B7, C7, D7, E7, F7, G7,
}

impl Square {
    /// N represents the total number of Squares in an Ataxx Board.
    pub const N: usize = File::N * Rank::N;

    /// new creates a new Square from the given File and Rank.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(Square::new(File::A, Rank::First), Square::A1);
    /// ```
    pub fn new(file: File, rank: Rank) -> Square {
        Square::unsafe_from(rank as usize * File::N + file as usize)
    }

    /// file returns the File of the current Square.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(Square::A1.file(), File::A);
    /// ```
    #[inline(always)]
    pub fn file(self) -> File {
        File::unsafe_from(self as usize % File::N)
    }

    /// rank returns the Rank of the current Square.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(Square::A1.rank(), Rank::First);
    /// ```
    #[inline(always)]
    pub fn rank(self) -> Rank {
        Rank::unsafe_from(self as usize / File::N)
    }

    /// north returns the Square to the North of this one.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(Square::D4.north(), Square::D5);
    /// ```
    pub fn north(self) -> Self {
        Square::unsafe_from(self as usize + File::N)
    }

    /// south returns the Square to the South of this one.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(Square::D4.south(), Square::D3);
    /// ```
    pub fn south(self) -> Self {
        Square::unsafe_from(self as usize - File::N)
    }

    /// east returns the Square to the East of this one.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(Square::D4.east(), Square::E4);
    /// ```
    pub fn east(self) -> Self {
        Square::unsafe_from(self as usize + 1)
    }

    /// west returns the Square to the West of this one.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(Square::D4.west(), Square::C4);
    /// ```
    pub fn west(self) -> Self {
        Square::unsafe_from(self as usize - 1)
    }
}

/// SquareParseError represents the various errors that can
/// be encountered while parsing a given string into a Square.
#[derive(Error, Debug)]
pub enum SquareParseError {
    #[error("wrong square string size")]
    WrongStringSize,
    #[error("{0}")]
    FileParseError(#[from] FileParseError),
    #[error("{0}")]
    RankParseError(#[from] RankParseError),
}

impl FromStr for Square {
    type Err = SquareParseError;

    /// from_str converts a square given in the format `<file><rank>` into
    /// a Square. For the formats of `<file>` and `<rank>` see the documentation
    /// for [`File::FromStr`](File::from_str) and [`Rank::FromStr`](Rank::from_str).
    /// It is effectively the inverse operation of Display.
    /// ```
    /// use ataxx::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Square::from_str("a1").unwrap(), Square::A1);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(SquareParseError::WrongStringSize);
        }

        let file = File::from_str(&s[..1])?; // Parse the File specification.
        let rank = Rank::from_str(&s[1..])?; // Parse the Rank specification.

        Ok(Square::new(file, rank))
    }
}

// Implement from and into traits for all primitive integer types.
type_macros::impl_from_integer_for_enum! {
    for Square Square::N =>

    // Unsigned Integers.
    usize, Square::from_usize;
    u8, Square::from_u8; u16, Square::from_u16;
    u32, Square::from_u32; u64, Square::from_u64;

    // Signed Integers.
    isize, Square::from_isize;
    i8, Square::from_i8; i16, Square::from_i16;
    i32, Square::from_i32; i64, Square::from_i64;
}

impl fmt::Display for Square {
    /// Display formats the given Square in the format `<file><rank>`. For how
    /// `<file>` and `<rank>` are formatted, see the documentation for
    /// `File::Display` and `Rank::Display` trait implementations.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(Square::A1.to_string(), "a1");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl fmt::Debug for Square {
    /// Debug implements Debug formatting for a Square.
    /// It uses `Square::Display` behind the hood.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// File represents a file on the Ataxx Board. Each vertical column of Squares
/// on an Ataxx Board is known as a File. There are 7 of them in total.
#[derive(Copy, Clone, PartialEq, PartialOrd, FromPrimitive, EnumIter)]
// @formatter:off
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
// @formatter:on

impl File {
    /// N represents the total number of Files in an Ataxx Board.
    pub const N: usize = 7;
}

/// FileParseError represents the various errors that can
/// be encountered while parsing a given string into a File.
#[derive(Error, Debug)]
pub enum FileParseError {
    #[error("wrong string size for file identifier")]
    WrongStringSize,
    #[error("invalid file identifier string")]
    InvalidFileString,
}

impl FromStr for File {
    type Err = FileParseError;

    /// from_str converts the given string representation of a File into its
    /// corresponding File value. String representations are lowercase alphabets
    ///  from a to g which represent Files from [`File::A`] to [`File::G`].
    /// ```
    /// use ataxx::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(File::from_str("a").unwrap(), File::A);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(FileParseError::WrongStringSize);
        }

        let ident = s.chars().next().unwrap() as u8;

        // File identifier should be one of a..h.
        if !(b'a'..=b'h').contains(&ident) {
            return Err(FileParseError::InvalidFileString);
        }

        Ok(File::unsafe_from(ident - b'a'))
    }
}

// Implement from and into traits for all primitive integer types.
type_macros::impl_from_integer_for_enum! {
    for File File::N =>

    // Unsigned Integers.
    usize, File::from_usize;
    u8, File::from_u8; u16, File::from_u16;
    u32, File::from_u32; u64, File::from_u64;

    // Signed Integers.
    isize, File::from_isize;
    i8, File::from_i8; i16, File::from_i16;
    i32, File::from_i32; i64, File::from_i64;
}

impl fmt::Display for File {
    /// Display formats the given File into a string. Specifically,
    /// it formats the File into a lowercase letter representing that File.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(File::A.to_string(), "a");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (b'a' + *self as u8) as char)
    }
}

impl fmt::Debug for File {
    /// Debug implements Debug formatting for a File.
    /// It uses `File::Display` behind the hood.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Rank represents a rank on the Ataxx Board. Each horizontal row of Squares
/// on an Ataxx Board is known as a Rank. There are 7 of them in total.
#[derive(Copy, Clone, PartialEq, PartialOrd, FromPrimitive, EnumIter)]
// @formatter:off
pub enum Rank {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
}
// @formatter:on

impl Rank {
    /// N represents the total number of Ranks in an Ataxx Board.
    pub const N: usize = 7;
}

/// RankParseError represents the various errors that can
/// be encountered while parsing a given string into a Rank.
#[derive(Error, Debug)]
pub enum RankParseError {
    #[error("wrong string size for rank identifier")]
    WrongStringSize,
    #[error("invalid rank identifier string")]
    InvalidRankString,
}

impl FromStr for Rank {
    type Err = RankParseError;

    /// from_str converts the given string representation of a Rank into its
    /// corresponding Rank value. String representations are single digit long
    /// decimal digits from 1 to 7 which represent the Ranks from
    /// [`Rank::First`] to [`Rank::Seventh`] respectively.
    /// ```
    /// use ataxx::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Rank::from_str("1").unwrap(), Rank::First);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(RankParseError::WrongStringSize);
        }

        let ident = s.chars().next().unwrap() as u8;

        // Rank identifier should be one of 1..8.
        if !(b'1'..=b'8').contains(&ident) {
            return Err(RankParseError::InvalidRankString);
        }

        Ok(Rank::unsafe_from(ident - b'1'))
    }
}

// Implement from and into traits for all primitive integer types.
type_macros::impl_from_integer_for_enum! {
    for Rank Rank::N =>

    // Unsigned Integers.
    usize, Rank::from_usize;
    u8, Rank::from_u8; u16, Rank::from_u16;
    u32, Rank::from_u32; u64, Rank::from_u64;

    // Signed Integers.
    isize, Rank::from_isize;
    i8, Rank::from_i8; i16, Rank::from_i16;
    i32, Rank::from_i32; i64, Rank::from_i64;
}

impl fmt::Display for Rank {
    /// Display formats the given Rank into a string. Specifically, it formats
    /// the Rank into a numerical digit from 1-7 representing that Rank.
    /// ```
    /// use ataxx::*;
    ///
    /// assert_eq!(Rank::First.to_string(), "1");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as usize + 1)
    }
}

impl fmt::Debug for Rank {
    /// Debug implements Debug formatting for a Rank.
    /// It uses `Rank::Display` behind the hood.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
