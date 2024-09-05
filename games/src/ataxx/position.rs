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

use crate::position_type;

use thiserror::Error;

use super::{BitBoard, Color, ColoredPiece, File, Hash, Move, Rank, Square};

use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use strum::IntoEnumIterator;

use crate::interface::{BitBoardType, PositionType, RepresentableType, SquareType, TypeParseError};

position_type! {
    struct Position {
        BitBoard = BitBoard
        ColoredPiece = ColoredPiece
        Move = Move

        self = self

        fn winner {
            if self.half_move_clock >= 100 {
                // Draw by 50 move rule.
                return None;
            }

            let black = self.colored_piece_bb(ColoredPiece::Black);
            let white = self.colored_piece_bb(ColoredPiece::White);
            let block = self.colored_piece_bb(ColoredPiece::Block);

            if black == BitBoard::EMPTY {
                // Black lost all its pieces, White won.
                return Some(Color::White);
            } else if white == BitBoard::EMPTY {
                // White lost all its pieces, Black won.
                return Some(Color::Black);
            }

            debug_assert!(black | white | block == BitBoard::UNIVERSE);

            let black_n = black.cardinality();
            let white_n = white.cardinality();

            match black_n.cmp(&white_n) {
                std::cmp::Ordering::Less => Some(Color::White),
                std::cmp::Ordering::Greater => Some(Color::Black),
                // Though there can't be an equal number of black and white pieces
                // on an empty ataxx board, it is possible with an odd number of
                // blocker pieces.
                std::cmp::Ordering::Equal => None,
            }
        }

        fn is_game_over {
            let black = self.colored_piece_bb(ColoredPiece::Black);
            let white = self.colored_piece_bb(ColoredPiece::White);
            let block = self.colored_piece_bb(ColoredPiece::Block);

            self.half_move_clock >= 100 ||                           // Fifty-move rule
                white | black | block == BitBoard::UNIVERSE ||       // All squares occupied
                white == BitBoard::EMPTY || black == BitBoard::EMPTY // No pieces left
        }

        fn after_move(UPDATE_HASH, m) {
            let stm = self.side_to_move;

            macro_rules! update_hash {
                ($e:expr) => {
                    if UPDATE_HASH {
                        $e
                    } else {
                        Hash::ZERO
                    }
                };
            }

            // A pass move is a do nothing move; just change the side to move.
            if m == Move::PASS {
                return Position {
                    bitboards: self.bitboards,
                    checksum: update_hash!(!self.checksum),
                    side_to_move: !self.side_to_move,
                    ply_count: self.ply_count + 1,
                    half_move_clock: self.half_move_clock + 1,
                };
            }

            let stm_pieces = self.color_bb(stm);
            let xtm_pieces = self.color_bb(!stm);

            let captured = crate::ataxx::moves::single(m.target()) & xtm_pieces;
            let from_to = BitBoard::from(m.target()) | BitBoard::from(m.source());

            // Move the captured pieces from xtm to stm.
            let new_xtm = xtm_pieces ^ captured;
            let new_stm = stm_pieces ^ captured ^ from_to;

            // Reset half move clock on a singular move.
            let half_move_clock = if m.is_single() {
                0
            } else {
                self.half_move_clock + 1
            };

            let (white, black) = if stm == Color::White {
                (new_stm, new_xtm)
            } else {
                (new_xtm, new_stm)
            };

            Position {
                bitboards: [black, white, self.colored_piece_bb(ColoredPiece::Block)],
                checksum: update_hash!(Hash::new(black, white, !stm)),
                side_to_move: !stm,
                ply_count: self.ply_count + 1,
                half_move_clock,
            }
        }

        fn generate_moves_into(QUIET, NOISY, T, movelist) {
            if self.is_game_over() {
                // Game is over, so don't generate any moves.
                return;
            }

            let stm = self.color_bb(self.side_to_move);
            let xtm = self.color_bb(!self.side_to_move);
            let gap = self.colored_piece_bb(ColoredPiece::Block);

            // Pieces can only move to unoccupied Squares.
            let allowed = !(stm | xtm | gap);

            for target in crate::ataxx::moves::singles(stm) & allowed {
                movelist.push(Move::new_single(target));
            }

            for piece in stm {
                // There may be multiple jump moves to a single Square, so they need to be
                // verified (& allowed) and serialized into the movelist immediately.
                let double = crate::ataxx::moves::double(piece) & allowed;
                for target in double {
                    movelist.push(Move::new(piece, target));
                }
            }

            // If there are no legal moves possible on the Position and the game isn't
            // over, a pass move is the only move possible to be played.
            if movelist.len() == 0 {
                movelist.push(Move::PASS);
            }
        }

        fn count_moves(QUIET, NOISY) {
            if self.is_game_over() {
                // Game is over, so don't generate any moves.
                return 0;
            }

            let stm = self.color_bb(self.side_to_move);
            let xtm = self.color_bb(!self.side_to_move);
            let gap = self.colored_piece_bb(ColoredPiece::Block);

            // Pieces can only move to unoccupied Squares.
            let allowed = !(stm | xtm | gap);

            // Count the number single moves in the Position.
            let mut moves: usize = (crate::ataxx::moves::singles(stm) & allowed).cardinality();

            for piece in stm {
                // There may be multiple jump moves to a single Square, so they need to be
                // verified (& allowed) and counted into the Position total immediately.
                let double = crate::ataxx::moves::double(piece) & allowed;
                moves += double.cardinality();
            }

            // If there are no legal moves possible on the Position and the game isn't
            // over, a pass move is the only move possible to be played.
            if moves == 0 {
                return 1;
            }

            moves
        }
    }
}

/// PositionParseErr represents an error encountered while parsing
/// the given FEN position field into a valid Position.
#[derive(Error, Debug)]
pub enum PositionParseError {
    #[error("expected 3 fields, found {0}")]
    TooManyFields(usize),

    #[error("a jump value was too long and overshot")]
    JumpTooLong,

    #[error("invalid piece identifier '{0}'")]
    InvalidPieceIdent(char),
    #[error("insufficient data to fill the entire {0} file")]
    FileDataIncomplete(File),
    #[error("expected 7 ranks, found more")]
    TooManyRanks,

    #[error("parsing side to move: {0}")]
    BadSideToMove(#[from] TypeParseError),
    #[error("parsing half-move clock: {0}")]
    BadHalfMoveClock(#[from] ParseIntError),
}

// FromStr implements parsing of the position field in a FEN.
impl FromStr for Position {
    type Err = PositionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect::<Vec<&str>>();

        if parts.len() != 4 {
            return Err(PositionParseError::TooManyFields(parts.len()));
        }

        let pos = parts[0];
        let stm = parts[1];
        let hmc = parts[2];
        let fmc = parts[3];

        let mut position = Position {
            bitboards: [BitBoard::EMPTY; ColoredPiece::N],
            checksum: Hash::ZERO,
            side_to_move: Color::Black,
            ply_count: 0,
            half_move_clock: 0,
        };

        // Spilt the position spec by the Ranks which are separated by '/'.
        let ranks: Vec<&str> = pos.split('/').collect();

        let mut rank = Ok(Rank::Seventh);
        let mut file = Ok(File::A);

        // Iterate over the Ranks in the string spec.
        for rank_data in ranks {
            // Rank pointer ran out, but data carried on.
            if rank.is_err() {
                return Err(PositionParseError::TooManyRanks);
            }

            // Iterate over the Square specs in the Rank spec.
            for data in rank_data.chars() {
                // Check if a jump spec was too big and we landed on an invalid File.
                if file.is_err() {
                    return Err(PositionParseError::JumpTooLong);
                }
                let square = Square::new(file.unwrap(), rank.unwrap());
                match data {
                    'o' | 'O' | 'w' | 'W' => position.insert(square, ColoredPiece::White),
                    'x' | 'X' | 'b' | 'B' => position.insert(square, ColoredPiece::Black),
                    '-' => position.insert(square, ColoredPiece::Block),

                    // Numbers represent jump specs to jump over empty squares.
                    '1'..='8' => {
                        file = File::try_from(file.unwrap() as u8 + data as u8 - b'1');
                        if file.is_err() {
                            return Err(PositionParseError::JumpTooLong);
                        }
                    }

                    _ => return Err(PositionParseError::InvalidPieceIdent(data)),
                }

                // On to the next Square spec in the Rank spec.
                file = File::try_from(file.unwrap() as u8 + 1);
            }

            // After rank data runs out, file pointer should be
            // at the last file, i.e, rank is completely filled.
            if let Ok(file) = file {
                return Err(PositionParseError::FileDataIncomplete(file));
            }

            // Switch rank pointer and reset file pointer.
            rank = Rank::try_from((rank.unwrap() as u8).wrapping_sub(1));
            file = Ok(File::A);
        }

        position.side_to_move = Color::from_str(stm)?;
        position.half_move_clock = hmc.parse::<u8>()?;
        position.ply_count = fmc.parse::<u16>()? * 2 - 1;
        if position.side_to_move == Color::Black {
            position.ply_count -= 1;
        }

        // Calculate the Hash value for the Position.
        position.checksum = Hash::new(
            position.colored_piece_bb(ColoredPiece::Black),
            position.colored_piece_bb(ColoredPiece::White),
            position.side_to_move,
        );

        Ok(position)
    }
}

// Display implements displaying a Position using ASCII art.
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board = self;
        let mut string_rep = String::from(" ");

        for rank in Rank::iter().rev() {
            for file in File::iter() {
                let square = Square::new(file, rank);
                let square_str = match board.at(square) {
                    Some(piece) => format!("{} ", piece),
                    None => ". ".to_string(),
                };
                string_rep += &square_str;
            }

            // Append the rank marker.
            string_rep += &format!(" {} \n ", rank);
        }

        // Append the file markers.
        string_rep += "a b c d e f g\n";

        writeln!(f, "{}", string_rep).unwrap();
        writeln!(f, "Side To Move: {}", self.side_to_move)
    }
}
