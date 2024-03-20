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

use std::{fmt::Display, ops, str::FromStr};

use crate::util::type_macros;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Copy, Clone, PartialEq, Eq, Default, FromPrimitive)]
#[rustfmt::skip]
pub enum Color {
    White, Black, #[default] None,
}

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
    pub const N: usize = 2;
}

impl ops::Not for Color {
    type Output = Color;
    fn not(self) -> Self::Output {
        Color::from(self as usize ^ 1)
    }
}

#[derive(Debug)]
pub enum ColorParseError {
    StringTooLong,
    StringFormatInvalid,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(ColorParseError::StringTooLong);
        }

        match s {
            "o" | "O" | "w" => Ok(Color::White),
            "x" | "X" | "b" => Ok(Color::Black),
            _ => Err(ColorParseError::StringFormatInvalid),
        }
    }
}
