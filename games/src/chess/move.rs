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

use crate::{
    chess,
    interface::{representable_type, MoveType, RepresentableType},
};

#[derive(Copy, Clone, PartialEq, Default)]
pub struct Move(u16);

impl MoveType for Move {
    const NULL: Move = Move(0);
    const MAX_IN_GAME: usize = 256;
    const MAX_IN_POSITION: usize = 256;
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

impl FromStr for Move {
    type Err = ();
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self::NULL)
    }
}

impl Move {
    // Bit-widths of fields.
    const SOURCE_WIDTH: u16 = 6;
    const TARGET_WIDTH: u16 = 6;
    const PROMOT_WIDTH: u16 = 2;
    const MVFLAG_WIDTH: u16 = 2;

    // Bit-masks of fields.
    const SOURCE_MASK: u16 = (1 << Move::SOURCE_WIDTH) - 1;
    const TARGET_MASK: u16 = (1 << Move::TARGET_WIDTH) - 1;
    const PROMOT_MASK: u16 = (1 << Move::PROMOT_WIDTH) - 1;
    const MVFLAG_MASK: u16 = (1 << Move::MVFLAG_WIDTH) - 1;

    // Bit-offsets of fields.
    const SOURCE_OFFSET: u16 = 0;
    const TARGET_OFFSET: u16 = Move::SOURCE_OFFSET + Move::SOURCE_WIDTH;
    const PROMOT_OFFSET: u16 = Move::TARGET_OFFSET + Move::TARGET_WIDTH;
    const MVFLAG_OFFSET: u16 = Move::PROMOT_OFFSET + Move::PROMOT_WIDTH;

    #[inline(always)]
    pub fn new(
        source: chess::Square,
        target: chess::Square,
        mvflag: MoveFlag,
    ) -> Move {
        Move(
            (mvflag as u16) << Move::MVFLAG_OFFSET
                | (source as u16) << Move::SOURCE_OFFSET
                | (target as u16) << Move::TARGET_OFFSET,
        )
    }

    #[inline(always)]
    pub fn new_with_promotion(
        source: chess::Square,
        target: chess::Square,
        promotion: chess::Piece,
    ) -> Move {
        Move(
            (promotion as u16 - 1) << Move::PROMOT_OFFSET
                | (MoveFlag::Promotion as u16) << Move::MVFLAG_OFFSET
                | (source as u16) << Move::SOURCE_OFFSET
                | (target as u16) << Move::TARGET_OFFSET,
        )
    }

    #[inline(always)]
    pub fn source(self) -> chess::Square {
        unsafe {
            chess::Square::unsafe_from(
                (self.0 >> Move::SOURCE_OFFSET) & Move::SOURCE_MASK,
            )
        }
    }

    #[inline(always)]
    pub fn target(self) -> chess::Square {
        unsafe {
            chess::Square::unsafe_from(
                (self.0 >> Move::TARGET_OFFSET) & Move::TARGET_MASK,
            )
        }
    }

    #[inline(always)]
    pub fn promot(self) -> chess::Piece {
        // +1 to account for the fact that move encodes
        // Piece::Knight as 0, while actually it is 1.
        unsafe {
            chess::Piece::unsafe_from(
                ((self.0 >> Move::PROMOT_OFFSET) & Move::PROMOT_MASK) + 1,
            )
        }
    }

    #[inline(always)]
    pub fn flags(self) -> MoveFlag {
        unsafe {
            MoveFlag::unsafe_from(
                ((self.0 >> Move::MVFLAG_OFFSET) & Move::MVFLAG_MASK) as u8,
            )
        }
    }
}

representable_type! {
    enum MoveFlag: u8 {
        Normal "n", Castle "c", Promotion "p", EnPassant "e",
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.source(), self.target())
    }
}
