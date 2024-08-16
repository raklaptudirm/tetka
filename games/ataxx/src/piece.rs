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

/// Color represents all the possible colors that an ataxx piece can have,
/// specifically, Black and White.
#[derive(Copy, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum Color {
    Black,
    White,
}

impl Color {
    /// N is the number of possible Colors.
    pub const N: usize = 2;

    #[inline(always)]
    pub fn unsafe_from<T: num_traits::ToPrimitive>(number: T) -> Self {
        debug_assert!(number.to_u64().unwrap() < Self::N as u64);
        unsafe { std::mem::transmute_copy(&number) }
    }
}

impl ops::Not for Color {
    type Output = Color;

    /// not implements the not unary operator (!) which switches the current Color
    /// to its opposite, i.e. [`Color::Black`] to [`Color::White`] and vice versa.
    fn not(self) -> Self::Output {
        Color::unsafe_from(self as usize ^ 1)
    }
}

#[derive(Error, Debug)]
pub enum ColorParseError {
    #[error("color identifier string has more than 1 character")]
    StringTooLong,
    #[error("unknown color identifier '{0}'")]
    StringFormatInvalid(String),
}

impl fmt::Display for Color {
    /// Implements displaying the Color in a human-readable form. [`Color::Black`]
    /// is formatted as `x` and [`Color::White`] is formatted as `o`.
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
    /// Debug implements debug printing of a Color in a human-readable form. It uses
    /// `Color::Display` under the hood to format and print the Color.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl FromStr for Color {
    type Err = ColorParseError;

    /// from_str converts the given human-readable string into its corresponding
    /// [`Color`]. `x`, `X`, `b`, `B` are parsed as [`Color::Black`] and `o`, `O`,
    /// `w`, `W` are parsed as [`Color::White`]. Best practice is to use `x` and `o`
    /// respectively for Black and White.
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

/// Piece represents all the possible ataxx pieces.
#[derive(Copy, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum Piece {
    Black,
    White,
    Block,
}

impl Piece {
    /// N is the number of possible Pieces.
    pub const N: usize = 3;

    pub fn new(color: Color) -> Self {
        match color {
            Color::Black => Piece::Black,
            Color::White => Piece::White,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Piece::Black => Color::Black,
            Piece::White => Color::White,
            _ => panic!("Piece::color() called on Piece::Block"),
        }
    }

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

impl fmt::Debug for Piece {
    /// Debug implements debug printing of a Piece in a human-readable form. It uses
    /// `Piece::Display` under the hood to format and print the Piece.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}