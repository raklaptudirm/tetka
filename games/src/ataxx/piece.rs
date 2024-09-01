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

use crate::interface::ColoredPieceType;
use crate::interface::RepresentableType;
use crate::representable_type;

representable_type!(
    /// Color represents all the possible colors that an ataxx piece can have,
    /// specifically, Black and White.
    enum Color: u8 { Black "x", White "o", }
);

impl ops::Not for Color {
    type Output = Color;

    /// not implements the not unary operator (!) which switches the current Color
    /// to its opposite, i.e. [`Color::Black`] to [`Color::White`] and vice versa.
    fn not(self) -> Self::Output {
        unsafe { Color::unsafe_from(self as usize ^ 1) }
    }
}

representable_type!(
    /// Piece represents all the possible ataxx pieces.
    enum ColoredPiece: u8 { Black "x", White "o", Block "■", }
);

impl ColoredPieceType for ColoredPiece {
    type Piece = Color;
    type Color = Color;

    fn piece(self) -> Color {
        match self {
            ColoredPiece::Black => Color::Black,
            ColoredPiece::White => Color::White,
            _ => panic!("Piece::color() called on Piece::Block"),
        }
    }

    fn color(self) -> Color {
        match self {
            ColoredPiece::Black => Color::Black,
            ColoredPiece::White => Color::White,
            _ => panic!("Piece::color() called on Piece::Block"),
        }
    }
}
