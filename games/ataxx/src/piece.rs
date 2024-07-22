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

use thiserror::Error;

/// Piece represents all the possible pieces that an ataxx piece can have,
/// specifically Black, White, and None(no Piece/no piece).
#[derive(Copy, Clone, PartialEq, Eq, Default, FromPrimitive)]
pub enum Piece {
    Black,
    White,
    Block,
    #[default]
    None,
}

impl Piece {
    /// N is the number of possible Pieces, excluding None.
    pub const N: usize = 3;

    #[inline(always)]
    pub fn unsafe_from<T: num_traits::ToPrimitive>(number: T) -> Self {
        debug_assert!(number.to_u64().unwrap() < (Self::N + 1) as u64);
        unsafe { std::mem::transmute_copy(&number) }
    }
}

impl ops::Not for Piece {
    type Output = Piece;

    /// not implements the not unary operator (!) which switches the current Piece
    /// to its opposite, i.e. [`Piece::Black`] to [`Piece::White`] and vice versa.
    fn not(self) -> Self::Output {
        Piece::unsafe_from(self as usize ^ 1)
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
                Self::None => ".",
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

impl FromStr for Piece {
    type Err = PieceParseError;

    /// from_str converts the given human-readable string into its corresponding
    /// [`Piece`]. `x`, `X`, `b`, `B` are parsed as [`Piece::Black`] and `o`, `O`,
    /// `w`, `W` are parsed as [`Piece::White`]. Best practice is to use `x` and `o`
    /// respectively for black and white.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(PieceParseError::StringTooLong);
        }

        match s {
            "x" | "X" | "b" | "B" => Ok(Piece::Black),
            "o" | "O" | "w" | "W" => Ok(Piece::White),
            "-" => Ok(Piece::Block),
            _ => Err(PieceParseError::StringFormatInvalid(s.to_string())),
        }
    }
}
