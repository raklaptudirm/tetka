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

use crate::interface::representable_type;
use crate::interface::ColoredPieceType;
use crate::interface::RepresentableType;

representable_type!(
    /// Color represents all the possible colors that an ataxx piece can have,
    /// specifically, Black and White.
    enum Color: u8 { White "w", Black "b", }
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
    /// Piece represents the types of pieces in ataxx, namely Piece and Block.
    enum Piece: u8 {
        Pawn "p", Knight "n", Bishop "b", Rook "r", Queen "q", King "k",
    }
);

representable_type!(
    /// Piece represents all the possible ataxx pieces.
    enum ColoredPiece: u8 {
        WhitePawn "P", WhiteKnight "N", WhiteBishop "B",
        WhiteRook "R", WhiteQueen "Q", WhiteKing "K",
        BlackPawn "p", BlackKnight "n", BlackBishop "b",
        BlackRook "r", BlackQueen "q", BlackKing "k",
    }
);

impl ColoredPieceType for ColoredPiece {
    type Piece = Piece;
    type Color = Color;

    fn piece(self) -> Piece {
        match self {
            ColoredPiece::BlackPawn | ColoredPiece::WhitePawn => Piece::Pawn,
            ColoredPiece::BlackKnight | ColoredPiece::WhiteKnight => {
                Piece::Knight
            }
            ColoredPiece::BlackBishop | ColoredPiece::WhiteBishop => {
                Piece::Bishop
            }
            ColoredPiece::BlackRook | ColoredPiece::WhiteRook => Piece::Rook,
            ColoredPiece::BlackQueen | ColoredPiece::WhiteQueen => Piece::Queen,
            ColoredPiece::BlackKing | ColoredPiece::WhiteKing => Piece::King,
        }
    }

    fn color(self) -> Color {
        match self {
            ColoredPiece::WhitePawn
            | ColoredPiece::WhiteKnight
            | ColoredPiece::WhiteBishop
            | ColoredPiece::WhiteRook
            | ColoredPiece::WhiteQueen
            | ColoredPiece::WhiteKing => Color::White,
            ColoredPiece::BlackPawn
            | ColoredPiece::BlackKnight
            | ColoredPiece::BlackBishop
            | ColoredPiece::BlackRook
            | ColoredPiece::BlackQueen
            | ColoredPiece::BlackKing => Color::Black,
        }
    }
}
