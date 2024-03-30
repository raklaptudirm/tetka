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
use std::fmt::Formatter;
use std::ops;
use std::str::FromStr;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::util::type_macros;

/// Color represents all the possible colors that an ataxx piece can have,
/// specifically White, Black, and None(no Color/no piece).
#[derive(Copy, Clone, PartialEq, Eq, Default, FromPrimitive)]
pub enum Color {
	White,
	Black,
	#[default] None,
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

// Implement not(!) operation to switch to the other Color.
impl ops::Not for Color {
	type Output = Color;
	fn not(self) -> Self::Output {
		Color::try_from(self as usize ^ 1).unwrap()
	}
}

#[derive(Debug)]
pub enum ColorParseError {
	StringTooLong,
	StringFormatInvalid,
}

impl fmt::Display for Color {
	/// Implements displaying the Color in a human-readable form.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match *self {
				Self::White => "o",
				Self::Black => "x",
				Self::None => "-",
			}
		)
	}
}

impl fmt::Debug for Color {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self)
	}
}

impl FromStr for Color {
	type Err = ColorParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.len() != 1 {
			return Err(ColorParseError::StringTooLong);
		}

		match s {
			"o" | "O" | "w" | "W" => Ok(Color::White),
			"x" | "X" | "b" | "B" => Ok(Color::Black),
			_ => Err(ColorParseError::StringFormatInvalid),
		}
	}
}
