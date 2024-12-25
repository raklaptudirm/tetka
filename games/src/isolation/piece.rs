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

use std::ops;

use crate::interface::ColoredPieceType;
use crate::interface::RepresentableType;
use crate::interface::{color_type, representable_type};

color_type!(
    /// Color represents all the possible colors that an ataxx piece can have,
    /// specifically, Black and White.
    enum Color { White "w", Black "b", }
);

representable_type!(
    /// Piece represents the types of pieces in ataxx, namely Piece and Block.
    enum Piece: u8 { Pawn "p", Tile "-", }
);

representable_type!(
    /// Piece represents all the possible ataxx pieces.
    enum ColoredPiece: u8 { WhitePawn "P", BlackPawn "p", Tile "-", }
);

impl ColoredPieceType for ColoredPiece {
    type Piece = Piece;
    type Color = Color;

    fn piece(self) -> Piece {
        match self {
            ColoredPiece::WhitePawn | ColoredPiece::BlackPawn => Piece::Pawn,
            ColoredPiece::Tile => Piece::Tile,
        }
    }

    fn color(self) -> Color {
        match self {
            ColoredPiece::WhitePawn => Color::White,
            ColoredPiece::BlackPawn => Color::Black,
            _ => panic!("Piece::color() called on Piece::Tile"),
        }
    }
}
