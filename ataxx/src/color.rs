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
use num_traits::FromPrimitive;

use thiserror::Error;

use crate::type_macros;

/// Color represents all the possible colors that an ataxx piece can have,
/// specifically White, Black, and None(no Color/no piece).
#[derive(Copy, Clone, PartialEq, Eq, Default, FromPrimitive)]
pub enum Color {
    White,
    Black,
    #[default]
    None,
}

// Implement conversions from numerical types.
type_macros::impl_from_integer_for_enum! {
    for Color:

    // unsigned integers
    usize, Color::from_usize;
    u8, Color::from_u8; u16, Color::from_u16;
    u32, Color::from_u32; u64, Color::from_u64;

    // signed integers
    isize, Color::from_isize;
    i8, Color::from_i8; i16, Color::from_i16;
    i32, Color::from_i32; i64, Color::from_i64;
}

impl Color {
    /// N is the number of possible Colors, excluding None.
    pub const N: usize = 2;
}

impl ops::Not for Color {
    type Output = Color;

    /// not implements the not unary operator (!) which switches the current Color
    /// to its opposite, i.e. [`Color::Black`] to [`Color::White`] and vice versa.
    fn not(self) -> Self::Output {
        Color::try_from(self as usize ^ 1).unwrap()
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
                Self::None => "-",
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
