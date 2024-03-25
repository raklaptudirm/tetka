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

use std::{fmt::Display, num::ParseIntError, str::FromStr};

use super::{
    Board, Color, ColorParseError, Position, PositionParseErr,
};

pub struct FEN {
    pub position: Position,
    pub half_move_clock: u8,
    pub full_move_count: u16,
}

impl FEN {
    const MAILBOX_OFFSET: usize = 0;
    const SIDE_TM_OFFSET: usize = 1;
    const HALF_MV_OFFSET: usize = 2;
    const FULL_MV_OFFSET: usize = 3;
}

impl From<&Board> for FEN {
    fn from(board: &Board) -> Self {
        FEN {
            position: board.position(),
            half_move_clock: 0,
            full_move_count: 0,//board.full_moves(),
        }
    }
}

impl Display for FEN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.position,
            self.half_move_clock,
            self.full_move_count
        )
    }
}

#[derive(Debug)]
pub enum FENParseError {
    WrongFieldNumber,
    PositionParseError(PositionParseErr),
    SideToMoveParseError(ColorParseError),
    HalfMoveClockParseError(ParseIntError),
    FullMoveClockParseError(ParseIntError),
}

impl FromStr for FEN {
    type Err = FENParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split fen into it's fields along the whitespace.
        let fields: Vec<&str> = s.split_whitespace().collect();

        // Verify the presence of the 6 fen fields.
        if fields.len() != 4 {
            return Err(FENParseError::WrongFieldNumber);
        }

        // Parse mailbox position representation.
        let mut position = match Position::from_str(fields[FEN::MAILBOX_OFFSET]) {
            Ok(position) => position,
            Err(err) => return Err(FENParseError::PositionParseError(err)),
        };

        // Parse side to move.
        position.side_to_move = match Color::from_str(fields[FEN::SIDE_TM_OFFSET]) {
            Ok(stm) => stm,
            Err(err) => return Err(FENParseError::SideToMoveParseError(err)),
        };

        // Parse half move clock.
        let half_move_clock = match str::parse::<u8>(fields[FEN::HALF_MV_OFFSET]) {
            Ok(half_move_clock) => half_move_clock,
            Err(err) => return Err(FENParseError::HalfMoveClockParseError(err)),
        };

        // Parse full move count.
        let full_move_count = match str::parse::<u16>(fields[FEN::FULL_MV_OFFSET]) {
            Ok(full_move_count) => full_move_count,
            Err(err) => return Err(FENParseError::FullMoveClockParseError(err)),
        };

        Ok(FEN {
            position,
            half_move_clock,
            full_move_count,
        })
    }
}
