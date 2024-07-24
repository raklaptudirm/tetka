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
use std::ops;
use std::str::FromStr;

use num_derive::FromPrimitive;

use num_derive::ToPrimitive;
use thiserror::Error;

/// Piece represents all the possible pieces that an ataxx piece can have,
/// specifically Black, White, and None(no Piece/no piece).
#[derive(Copy, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum Color {
    Black,
    White,
}

impl Color {
    /// N is the number of possible Pieces, excluding None.
    pub const N: usize = 2;

    #[inline(always)]
    pub fn unsafe_from<T: num_traits::ToPrimitive>(number: T) -> Self {
        debug_assert!(number.to_u64().unwrap() < Self::N as u64);
        unsafe { std::mem::transmute_copy(&number) }
    }
}

impl ops::Not for Color {
    type Output = Color;

    /// not implements the not unary operator (!) which switches the current Piece
    /// to its opposite, i.e. [`Piece::Black`] to [`Piece::White`] and vice versa.
    fn not(self) -> Self::Output {
        Color::unsafe_from(self as usize ^ 1)
    }
}

impl From<Color> for Piece {
    fn from(value: Color) -> Self {
        Piece::unsafe_from(value)
    }
}

#[derive(Error, Debug)]
pub enum ColorParseError {
    #[error("piece identifier string has more than 1 character")]
    StringTooLong,
    #[error("unknown piece identifier '{0}'")]
    StringFormatInvalid(String),
}

impl fmt::Display for Color {
    /// Implements displaying the Piece in a human-readable form. [`Piece::Black`]
    /// is formatted as `x` and [`Piece::White`] is formatted as `o`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Black => "x",
                Self::White => "o",
            }
        )
    }
}

impl fmt::Debug for Color {
    /// Debug implements debug printing of a Piece in a human-readable form. It uses
    /// `Piece::Display` under the hood to format and print the Piece.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl FromStr for Color {
    type Err = ColorParseError;

    /// from_str converts the given human-readable string into its corresponding
    /// [`Piece`]. `x`, `X`, `b`, `B` are parsed as [`Piece::Black`] and `o`, `O`,
    /// `w`, `W` are parsed as [`Piece::White`]. Best practice is to use `x` and `o`
    /// respectively for black and white.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(ColorParseError::StringTooLong);
        }

        match s {
            "x" | "X" | "b" | "B" => Ok(Color::Black),
            "o" | "O" | "w" | "W" => Ok(Color::White),
            _ => Err(ColorParseError::StringFormatInvalid(s.to_string())),
        }
    }
}

/// Piece represents all the possible pieces that an ataxx piece can have,
/// specifically Black, White, and None(no Piece/no piece).
#[derive(Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub enum Piece {
    Black,
    White,
    Block,
}

impl Piece {
    /// N is the number of possible Pieces, excluding None.
    pub const N: usize = 3;

    #[inline(always)]
    pub fn unsafe_from<T: num_traits::ToPrimitive>(number: T) -> Self {
        debug_assert!(number.to_u64().unwrap() < Self::N as u64);
        unsafe { std::mem::transmute_copy(&number) }
    }
}

#[derive(Error, Debug)]
pub enum PieceParseError {
    #[error("piece identifier string has more than 1 character")]
    StringTooLong,
    #[error("unknown piece identifier '{0}'")]
    StringFormatInvalid(String),
}

impl fmt::Display for Piece {
    /// Implements displaying the Piece in a human-readable form. [`Piece::Black`]
    /// is formatted as `x` and [`Piece::White`] is formatted as `o`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Black => "x",
                Self::White => "o",
                Self::Block => "■",
            }
        )
    }
}
