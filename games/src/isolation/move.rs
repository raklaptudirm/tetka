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
use std::str::FromStr;

use thiserror::Error;

use super::Square;
use crate::interface::{MoveType, RepresentableType, TypeParseError};

/// Move represents an Ataxx move which can be played on the Board.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Move(u16);

impl MoveType for Move {
    const NULL: Self = Move(1 << 15);
    const MAX_IN_GAME: usize = 48;
    const MAX_IN_POSITION: usize = 352;
}

impl From<u16> for Move {
    fn from(value: u16) -> Self {
        Move(value)
    }
}

impl From<Move> for u16 {
    fn from(value: Move) -> Self {
        value.0
    }
}

impl Move {
    // Bit-widths of fields.
    const PAWN_WIDTH: u16 = 6;
    const TILE_WIDTH: u16 = 6;

    // Bit-masks of fields.
    const PAWN_MASK: u16 = (1 << Move::PAWN_WIDTH) - 1;
    const TILE_MASK: u16 = (1 << Move::TILE_WIDTH) - 1;

    // Bit-offsets of fields.
    const PAWN_OFFSET: u16 = 0;
    const TILE_OFFSET: u16 = Move::PAWN_OFFSET + Move::PAWN_WIDTH;

    /// NULL Move represents an invalid move.
    pub const NULL: Move = Move(1 << 15);

    /// new_single returns a new singular Move, where a piece is cloned to its
    /// target Square. For a singular Move, [`Move::source`] and [`Move::target`]
    /// are equal since the source Square is irrelevant to the Move.
    /// ```
    /// use tetka_games::ataxx::*;
    ///
    /// let mov = Move::new_single(Square::A1);
    ///
    /// assert_eq!(mov.source(), mov.target());
    /// assert_eq!(mov.target(), Square::A1);
    /// ```
    #[inline(always)]
    pub fn new_single(square: Square) -> Move {
        Move::new(square, square)
    }

    /// new returns a new jump Move from the given source Square to the given
    /// target Square. These Squares can be recovered with the [`Move::source`] and
    /// [`Move::target`] methods respectively.
    /// ```
    /// use tetka_games::ataxx::*;
    ///
    /// let mov = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(mov.source(), Square::A1);
    /// assert_eq!(mov.target(), Square::A3);
    /// ```
    #[inline(always)]
    #[rustfmt::skip]
    pub fn new(source: Square, target: Square) -> Move {
		Move(
			(source as u16) << Move::PAWN_OFFSET |
			(target as u16) << Move::TILE_OFFSET
		)
    }

    /// Source returns the source Square of the moving piece. This is equal to the
    /// target Square if the given Move is of singular type.
    /// ```
    /// use tetka_games::ataxx::*;
    ///
    /// let mov = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(mov.source(), Square::A1);
    /// ```
    pub fn pawn(self) -> Square {
        unsafe {
            Square::unsafe_from((self.0 >> Move::PAWN_OFFSET) & Move::PAWN_MASK)
        }
    }

    /// Target returns the target Square of the moving piece.
    /// ```
    /// use tetka_games::ataxx::*;
    ///
    /// let mov = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(mov.target(), Square::A3);
    /// ```
    pub fn tile(self) -> Square {
        unsafe {
            Square::unsafe_from((self.0 >> Move::TILE_OFFSET) & Move::TILE_MASK)
        }
    }
}

#[derive(Error, Debug)]
pub enum MoveParseError {
    #[error("length of move string should be 2 or 4, not {0}")]
    BadLength(usize),
    #[error("bad source square string \"{0}\"")]
    BadSquare(#[from] TypeParseError),
}

impl FromStr for Move {
    type Err = MoveParseError;

    /// from_str converts the given string representation of a Move into a [Move].
    /// The formats supported are '0000' for a [Move::PASS], `<target>` for a
    /// singular Move, and `<source><target>` for a jump Move. For how `<source>`
    /// and `<target>` are parsed, take a look at
    /// [`Square::FromStr`](Square::from_str). This function can be treated as the
    /// inverse of the [`fmt::Display`] trait for [Move].
    /// ```
    /// use tetka_games::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let pass = Move::PASS;
    /// let sing = Move::new_single(Square::A1);
    /// let jump = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(Move::from_str(&pass.to_string()).unwrap(), pass);
    /// assert_eq!(Move::from_str(&sing.to_string()).unwrap(), sing);
    /// assert_eq!(Move::from_str(&jump.to_string()).unwrap(), jump);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 && s.len() != 4 {
            return Err(MoveParseError::BadLength(s.len()));
        }

        let source = &s[..2];
        let source = Square::from_str(source)?;

        if s.len() < 4 {
            return Ok(Move::new_single(source));
        }

        let target = &s[2..];
        let target = Square::from_str(target)?;

        Ok(Move::new(source, target))
    }
}

impl fmt::Display for Move {
    /// Display formats the given Move in a human-readable manner. The format used
    /// for displaying jump moves is `<source><target>`, while a singular Move is
    /// formatted as `<target>`. For the formatting of `<source>` and `<target>`,
    /// refer to `Square::Display`. [`Move::NULL`] is  formatted as `null`, while
    /// [`Move::PASS`] is formatted as `0000`.
    /// ```
    /// use tetka_games::ataxx::*;
    ///
    /// let null = Move::NULL;
    /// let pass = Move::PASS;
    /// let sing = Move::new_single(Square::A1);
    /// let jump = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(null.to_string(), "null");
    /// assert_eq!(pass.to_string(), "0000");
    /// assert_eq!(sing.to_string(), "a1");
    /// assert_eq!(jump.to_string(), "a1a3");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Move::NULL {
            write!(f, "null")
        } else {
            write!(f, "{}{}", self.pawn(), self.tile())
        }
    }
}

impl fmt::Debug for Move {
    /// Debug formats the given Move into a human-readable debug string. It uses
    /// `Move::Display` trait under the hood for formatting the Move.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
