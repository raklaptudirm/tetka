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

use std::ops::Not;

use super::RepresentableType;

/// The ColoredPiece trait should be implemented by the piece representation
/// (with color) for a game.
pub trait ColoredPieceType: RepresentableType<u8>
where
    Self::Piece: RepresentableType<u8>,
    Self::Color: ColorType,
{
    /// The type for the Piece of the ColoredPiece.
    type Piece;
    /// The type for the Color of the ColoredPiece.
    type Color;

    /// Creates a new ColoredPiece from the given Piece and Color.
    #[must_use]
    fn new(piece: Self::Piece, color: Self::Color) -> Self {
        unsafe {
            Self::unsafe_from(
                color.into() * Self::Piece::N as u8 + piece.into(),
            )
        }
    }

    /// Returns the Piece of the given ColoredPiece.
    #[must_use]
    fn piece(self) -> Self::Piece;
    /// Returns the Color of the given ColoredPiece.
    #[must_use]
    fn color(self) -> Self::Color;
}

/// A type representing the color in a board representation.
pub trait ColorType: RepresentableType<u8> + Not {
    const FIRST: Self;
}
