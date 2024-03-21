// Copyright © 2023 Rak Laptudirm <rak@laptudirm.com>
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

use std::fmt::Display;
use std::str::FromStr;

use crate::ataxx::{self, Color};
use crate::util::type_macros;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use strum_macros::EnumIter;

/// Enum Square represents all the different squares on an ataxxboard.
#[derive(Copy, Clone, PartialEq, PartialOrd, FromPrimitive, EnumIter)]
#[rustfmt::skip]
pub enum Square {
    A1 = 0x00, B1, C1, D1, E1, F1, G1,
    A2 = 0x08, B2, C2, D2, E2, F2, G2,
    A3 = 0x10, B3, C3, D3, E3, F3, G3,
    A4 = 0x18, B4, C4, D4, E4, F4, G4,
    A5 = 0x20, B5, C5, D5, E5, F5, G5,
    A6 = 0x28, B6, C6, D6, E6, F6, G6,
    A7 = 0x30, B7, C7, D7, E7, F7, G7,
}

impl Square {
    /// N is the number of different squares.
    pub const N: usize = 64;

    pub fn new(file: File, rank: Rank) -> Square {
        Square::try_from(rank as usize * 8 + file as usize).unwrap()
    }

    #[inline(always)]
    pub fn file(self) -> File {
        File::try_from(self as usize % 8).unwrap()
    }

    #[inline(always)]
    pub fn rank(self) -> Rank {
        Rank::try_from(self as usize / 8).unwrap()
    }

    pub fn relative(self, color: ataxx::Color) -> Self {
        if color == ataxx::Color::White {
            self
        } else {
            self.flip_rank()
        }
    }

    pub fn flip_file(self) -> Self {
        // Flip the file bits.
        Self::try_from(self as usize ^ 0b_000_111).unwrap()
    }

    pub fn flip_rank(self) -> Self {
        // Flip the rank bits.
        Self::try_from(self as usize ^ 0b_111_000).unwrap()
    }

    pub fn up(self, us: Color) -> Self {
        match us {
            Color::White => self.north(),
            Color::Black => self.south(),
            Color::None => self,
        }
    }

    pub fn down(self, us: Color) -> Self {
        match us {
            Color::White => self.south(),
            Color::Black => self.north(),
            Color::None => self,
        }
    }

    pub fn north(self) -> Self {
        Square::try_from(self as usize - 8).unwrap()
    }

    pub fn south(self) -> Self {
        Square::try_from(self as usize + 8).unwrap()
    }

    pub fn east(self) -> Self {
        Square::try_from(self as usize + 1).unwrap()
    }

    pub fn west(self) -> Self {
        Square::try_from(self as usize - 1).unwrap()
    }

    pub fn distance(self, rhs: Square) -> usize {
        let rank_dist = (self.rank() as i32 - rhs.rank() as i32).unsigned_abs() as usize;
        let file_dist = (self.file() as i32 - rhs.file() as i32).unsigned_abs() as usize;

        rank_dist.max(file_dist)
    }
}

pub enum SquareParseError {
    WrongStringSize,
    FileParseError(FileParseError),
    RankParseError(RankParseError),
}

impl FromStr for Square {
    type Err = SquareParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(SquareParseError::WrongStringSize);
        }

        let file = match File::from_str(&s[..=0]) {
            Ok(file) => file,
            Err(err) => return Err(SquareParseError::FileParseError(err)),
        };

        let rank = match Rank::from_str(&s[1..]) {
            Ok(rank) => rank,
            Err(err) => return Err(SquareParseError::RankParseError(err)),
        };

        Ok(Square::new(file, rank))
    }
}

// Implement from and into traits for all primitive integer types.
type_macros::impl_from_integer_for_enum! {
    for Square:

    // Unsigned Integers.
    usize, Square::from_usize;
    u8, Square::from_u8; u16, Square::from_u16;
    u32, Square::from_u32; u64, Square::from_u64;

    // Signed Integers.
    isize, Square::from_isize;
    i8, Square::from_i8; i16, Square::from_i16;
    i32, Square::from_i32; i64, Square::from_i64;
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, FromPrimitive, EnumIter)]
#[rustfmt::skip]
pub enum File {
    A, B, C, D, E, F, G
}

impl File {
    pub const N: usize = 8;
    pub fn relative(self, color: ataxx::Color) -> File {
        match color {
            ataxx::Color::White => self,
            ataxx::Color::Black => File::try_from(6 - self as usize).unwrap(),
            ataxx::Color::None => panic!("Color::None"),
        }
    }
}

pub enum FileParseError {
    WrongStringSize,
    InvalidFileString,
}

impl FromStr for File {
    type Err = FileParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(FileParseError::WrongStringSize);
        }

        let ident = s.chars().next().unwrap() as u8;

        // File identifier should be one of a..h.
        if !(b'a'..=b'h').contains(&ident) {
            return Err(FileParseError::InvalidFileString);
        }

        Ok(File::try_from(ident - b'a').unwrap())
    }
}

// Implement from and into traits for all primitive integer types.
type_macros::impl_from_integer_for_enum! {
    for File:

    // Unsigned Integers.
    usize, File::from_usize;
    u8, File::from_u8; u16, File::from_u16;
    u32, File::from_u32; u64, File::from_u64;

    // Signed Integers.
    isize, File::from_isize;
    i8, File::from_i8; i16, File::from_i16;
    i32, File::from_i32; i64, File::from_i64;
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (b'a' + *self as u8) as char)
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, FromPrimitive, EnumIter)]
#[rustfmt::skip]
pub enum Rank {
    First, Second, Third, Fourth, Fifth, Sixth, Seventh
}

impl Rank {
    pub const N: usize = 8;
    pub fn relative(self, color: ataxx::Color) -> Rank {
        match color {
            ataxx::Color::White => self,
            ataxx::Color::Black => Rank::try_from(7 - self as usize).unwrap(),
            ataxx::Color::None => panic!("Color::None"),
        }
    }
}

pub enum RankParseError {
    WrongStringSize,
    InvalidRankString,
}

impl FromStr for Rank {
    type Err = RankParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(RankParseError::WrongStringSize);
        }

        let ident = s.chars().next().unwrap() as u8;

        // Rank identifier should be one of 1..8.
        if !(b'1'..=b'8').contains(&ident) {
            return Err(RankParseError::InvalidRankString);
        }

        Ok(Rank::try_from(7 - (ident - b'1')).unwrap())
    }
}

// Implement from and into traits for all primitive integer types.
type_macros::impl_from_integer_for_enum! {
    for Rank:

    // Unsigned Integers.
    usize, Rank::from_usize;
    u8, Rank::from_u8; u16, Rank::from_u16;
    u32, Rank::from_u32; u64, Rank::from_u64;

    // Signed Integers.
    isize, Rank::from_isize;
    i8, Rank::from_i8; i16, Rank::from_i16;
    i32, Rank::from_i32; i64, Rank::from_i64;
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as usize + 1)
    }
}
