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
use std::num::ParseIntError;
use std::str::FromStr;

use super::{Board, Color, Position};
use super::{ColorParseError, PositionParseErr};

/// FEN represents a parsed Forsyth Edwards Notation string. The original FEN
/// format was created for chess was it was adapted to be usable in ataxx.
///
/// # Format
/// `<position> <side to move> <half move clock> <full move count>`
///
/// ### `<position>`
/// This field specifies the piece configuration of the ataxx position. x/X/b/B
/// characters represent black pieces while o/O/w/W characters represent
/// white pieces on the board. Ranks are separated by /s and empty squares are
/// specified by a number which instructs how many squares to skip over.
///
/// ### `<side to move>`
/// This field specifies the current Color to move. The format for specifying
/// Colors is the same as the one used in position for specifying pieces.
///
/// ### `<half move clock>`
/// Records the number of plys since the last singular move. Once this counter
/// reaches 100, a draw can be claimed by the fifty-move rule.
///
/// ### `<full move count>`
/// Records the number of full moves (one black one white) made since the game
/// started. This field is not very essential but kept for compatibility.
pub struct FEN {
	pub position: Position,
	pub full_move_count: u16,
}

impl FEN {
	/// STARTPOS is the FEN string representing the ataxx starting position.
	pub const STARTPOS: &'static str = "x5o/7/7/7/7/7/o5x x 0 1";
}

// Offsets of different fields in the field array.
impl FEN {
	const MAILBOX_OFFSET: usize = 0;
	const SIDE_TM_OFFSET: usize = 1;
	const HALF_MV_OFFSET: usize = 2;
	const FULL_MV_OFFSET: usize = 3;
}

impl From<&Board> for FEN {
	/// From creates a new FEN from the given Board.
	fn from(board: &Board) -> Self {
		FEN {
			position: board.position(),

			// TODO: record full move counts
			full_move_count: 0,
		}
	}
}

impl fmt::Display for FEN {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{} {}",
			self.position,
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

		// Verify the presence of the correct number fen fields.
		if fields.len() < 2 || fields.len() > 4 {
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

		// Parse half move clock, if present.
		position.half_move_clock = if fields.len() > FEN::HALF_MV_OFFSET {
			match str::parse::<u8>(fields[FEN::HALF_MV_OFFSET]) {
				Ok(half_move_clock) => half_move_clock,
				Err(err) => return Err(FENParseError::HalfMoveClockParseError(err)),
			}
		} else {
			0 // Field is optional, default to 0.
		};

		// Parse full move count, if present.
		let full_move_count = if fields.len() > FEN::FULL_MV_OFFSET {
			match str::parse::<u16>(fields[FEN::FULL_MV_OFFSET]) {
				Ok(full_move_count) => full_move_count,
				Err(err) => return Err(FENParseError::FullMoveClockParseError(err)),
			}
		} else {
			1 // Field is optional, default to 1.
		};

		Ok(FEN {
			position,
			full_move_count,
		})
	}
}
