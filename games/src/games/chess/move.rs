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

use std::{fmt, str::FromStr};

use super::{castling, Piece, Position, Square};
use crate::{
    games::chess::Direction,
    interface::{
        representable_type, ColoredPieceType, MoveType, PositionType,
        RepresentableType, SquareType, TypeParseError,
    },
};

use thiserror::Error;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct Move(u16);

impl MoveType for Move {
    const NULL: Move = Move(0);
    const MAX_IN_GAME: usize = 256;
    const MAX_IN_POSITION: usize = 256;

    type Position = Position;
    type MoveParseError = MoveParseError;

    fn from_str(
        move_str: &str,
        position: &Self::Position,
    ) -> Result<Self, Self::MoveParseError> {
        if move_str.len() < 4 || move_str.len() > 5 {
            return Err(MoveParseError::BadLength(move_str.len()));
        }

        let source = Square::from_str(&move_str[0..2])?;
        let target = Square::from_str(&move_str[2..4])?;

        let source_piece = match position.at(source) {
            Some(piece) => piece.piece(),
            None => return Err(MoveParseError::EmptySource),
        };

        let is_pawn = source_piece == Piece::Pawn;

        let x_dist = (source.file() as u8).abs_diff(target.file() as u8);
        let y_dist = (source.rank() as u8).abs_diff(target.rank() as u8);

        let flag = if move_str.len() == 5 {
            MoveFlag::from_str(&move_str[4..])?
        } else if is_pawn && y_dist == 2 {
            MoveFlag::DoublePush
        } else if is_pawn && position.en_passant_target() == Some(target) {
            MoveFlag::EnPassant
        } else if source_piece == Piece::King && x_dist > 1 {
            if source as u8 > target as u8 {
                MoveFlag::CastleASide
            } else {
                MoveFlag::CastleHSide
            }
        } else {
            MoveFlag::Normal
        };

        Ok(Move::new(source, target, flag))
    }

    fn fmt(
        &self,
        position: &Self::Position,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self.flag() {
            promotion @ (MoveFlag::NPromotion
            | MoveFlag::BPromotion
            | MoveFlag::RPromotion
            | MoveFlag::QPromotion) => {
                write!(f, "{}{}{}", self.source(), self.target(), promotion)
            }
            castling @ (MoveFlag::CastleHSide | MoveFlag::CastleASide) => {
                if position.is_frc() {
                    write!(f, "{}{}", self.source(), self.target())
                } else {
                    let source = self.source();
                    let target = match castling {
                        MoveFlag::CastleHSide => {
                            source.shift(Direction::East).shift(Direction::East)
                        }
                        MoveFlag::CastleASide => {
                            source.shift(Direction::West).shift(Direction::West)
                        }
                        _ => unreachable!(),
                    };
                    write!(f, "{}{}", source, target)
                }
            }
            _ => write!(f, "{}{}", self.source(), self.target()),
        }
    }
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
    const SOURCE_WIDTH: u16 = 6;
    const TARGET_WIDTH: u16 = 6;
    const MVFLAG_WIDTH: u16 = 4;

    // Bit-masks of fields.
    const SOURCE_MASK: u16 = (1 << Move::SOURCE_WIDTH) - 1;
    const TARGET_MASK: u16 = (1 << Move::TARGET_WIDTH) - 1;
    const MVFLAG_MASK: u16 = (1 << Move::MVFLAG_WIDTH) - 1;

    // Bit-offsets of fields.
    const SOURCE_OFFSET: u16 = 0;
    const TARGET_OFFSET: u16 = Move::SOURCE_OFFSET + Move::SOURCE_WIDTH;
    const MVFLAG_OFFSET: u16 = Move::TARGET_OFFSET + Move::TARGET_WIDTH;

    pub fn new(source: Square, target: Square, mvflag: MoveFlag) -> Move {
        Move(
            (mvflag as u16) << Move::MVFLAG_OFFSET
                | (source as u16) << Move::SOURCE_OFFSET
                | (target as u16) << Move::TARGET_OFFSET,
        )
    }

    pub fn new_castling(
        king: Square,
        rook: Square,
        side: castling::Side,
    ) -> Move {
        Self::new(
            king,
            rook,
            match side {
                castling::Side::H => MoveFlag::CastleHSide,
                castling::Side::A => MoveFlag::CastleASide,
            },
        )
    }

    pub fn new_with_promotion(
        source: Square,
        target: Square,
        promotion: Piece,
    ) -> Move {
        Move(
            (promotion as u16) << Move::MVFLAG_OFFSET
                | (source as u16) << Move::SOURCE_OFFSET
                | (target as u16) << Move::TARGET_OFFSET,
        )
    }

    pub fn source(self) -> Square {
        unsafe {
            Square::unsafe_from(
                (self.0 >> Move::SOURCE_OFFSET) & Move::SOURCE_MASK,
            )
        }
    }

    pub fn target(self) -> Square {
        unsafe {
            Square::unsafe_from(
                (self.0 >> Move::TARGET_OFFSET) & Move::TARGET_MASK,
            )
        }
    }

    pub fn flag(self) -> MoveFlag {
        unsafe {
            MoveFlag::unsafe_from(
                ((self.0 >> Move::MVFLAG_OFFSET) & Move::MVFLAG_MASK) as u8,
            )
        }
    }
}

representable_type! {
    enum MoveFlag: u8 {
        Normal "N",
        NPromotion "n", BPromotion "b", RPromotion "r", QPromotion "q",
        EnPassant "e", DoublePush "d",
        CastleHSide "h", CastleASide "a",
    }
}

impl MoveFlag {
    /// # Safety
    /// This function can only be called safely if `self` is one of `NPromotion`,
    /// `BPromotion`, `RPromotion`, and `QPromotion`.
    pub unsafe fn promoted_piece(&self) -> Piece {
        Piece::unsafe_from(*self as usize)
    }

    pub fn is_promotion(&self) -> bool {
        matches!(
            self,
            MoveFlag::NPromotion
                | MoveFlag::BPromotion
                | MoveFlag::RPromotion
                | MoveFlag::QPromotion
        )
    }

    pub fn is_castling(&self) -> bool {
        matches!(self, MoveFlag::CastleHSide | MoveFlag::CastleASide)
    }
}

#[derive(Error, Debug)]
pub enum MoveParseError {
    #[error("length of move string should be 4 or 5, not {0}")]
    BadLength(usize),
    #[error("bad source square: {0}")]
    BadSquare(#[from] TypeParseError),
    #[error("source square for the move is empty")]
    EmptySource,
}
