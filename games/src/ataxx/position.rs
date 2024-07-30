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

use std::cmp;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use num_traits::FromPrimitive;
use strum::IntoEnumIterator;

use crate::interface::PositionType;
use crate::interface::{BitBoardType, RepresentableType, SquareType};

use thiserror::Error;

#[rustfmt::skip]
use crate::ataxx::{
    BitBoard, ColoredPiece, File, Hash, Move,
    Rank, Square,
    ColorParseError, Color
};
use crate::interface::MoveStore;

use super::moves;

/// Position represents the snapshot of an Ataxx Board, the state of the an
/// ataxx game at a single point in time. It also provides all of the methods
/// necessary to manipulate such a snapshot.
#[derive(Copy, Clone)]
pub struct Position {
    /// bitboards stores [BitBoard]s for the piece configuration of each [Piece].
    pub bitboards: [BitBoard; ColoredPiece::N],
    /// checksum stores the semi-unique [struct@Hash] of the current Position.
    pub checksum: Hash,
    /// side_to_move stores the [Piece] whose turn to move it currently is.
    pub side_to_move: Color,
    pub ply_count: u16,
    /// half-move clock stores the number of half-moves since the last irreversible
    /// Move. It is used to adjudicate games using the 50-move/100-ply rule.
    pub half_move_clock: u8,
}

impl PositionType for Position {
    type BitBoard = BitBoard;
    type ColoredPiece = ColoredPiece;
    type Move = Move;

    /// put puts the given piece represented by its Piece on the given Square.
    fn insert(&mut self, sq: Square, piece: ColoredPiece) {
        self.bitboards[piece as usize].insert(sq);
    }

    fn remove(&mut self, sq: Square) -> Option<ColoredPiece> {
        match self.at(sq) {
            Some(piece) => {
                self.bitboards[piece as usize].remove(sq);
                Some(piece)
            }
            None => None,
        }
    }

    /// at returns the Piece of the piece present on the given Square.
    fn at(&self, sq: Square) -> Option<ColoredPiece> {
        if self.colored_piece_bb(ColoredPiece::Block).contains(sq) {
            Some(ColoredPiece::Block)
        } else if self.colored_piece_bb(ColoredPiece::Black).contains(sq) {
            Some(ColoredPiece::Black)
        } else if self.colored_piece_bb(ColoredPiece::White).contains(sq) {
            Some(ColoredPiece::White)
        } else {
            None
        }
    }

    fn piece_bb(&self, piece: Color) -> BitBoard {
        self.bitboards[piece as usize]
    }

    fn color_bb(&self, color: Color) -> BitBoard {
        self.bitboards[color as usize]
    }

    fn colored_piece_bb(&self, piece: ColoredPiece) -> BitBoard {
        self.bitboards[piece as usize]
    }

    /// is_game_over checks if the game is over, i.e. is a win or a draw.
    fn is_game_over(&self) -> bool {
        let black = self.colored_piece_bb(ColoredPiece::Black);
        let white = self.colored_piece_bb(ColoredPiece::White);
        let block = self.colored_piece_bb(ColoredPiece::Block);

        self.half_move_clock >= 100 ||                           // Fifty-move rule
			white | black | block == BitBoard::UNIVERSE ||       // All squares occupied
			white == BitBoard::EMPTY || black == BitBoard::EMPTY // No pieces left
    }

    /// winner returns the Piece which has won the game. It returns [`None`]
    /// if the game is a draw. If [`Position::is_game_over`] is false, then the
    /// behavior of this function is undefined.
    fn winner(&self) -> Option<Color> {
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

        // All the squares are occupied by pieces. Victory is decided by
        // which Piece has the most number of pieces on the Board.

        let black_n = black.cardinality();
        let white_n = white.cardinality();

        match black_n.cmp(&white_n) {
            cmp::Ordering::Less => Some(Color::White),
            cmp::Ordering::Greater => Some(Color::Black),
            // Though there can't be an equal number of black and white pieces
            // on an empty ataxx board, it is possible with an odd number of
            // blocker pieces.
            cmp::Ordering::Equal => None,
        }
    }

    /// after_move returns a new Position which occurs when the given Move is
    /// played on the current Position. Its behavior is undefined if the given
    /// Move is illegal.
    fn after_move<const UPDATE_HASH: bool>(&self, m: Move) -> Position {
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

        let captured = moves::single(m.target()) & xtm_pieces;
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

    /// generate_moves_into generates all the legal moves in the current Position
    /// and adds them to the given movelist. The type of the movelist must
    /// implement the [`MoveStore`] trait.
    fn generate_moves_into<const QUIET: bool, const NOISY: bool, T: MoveStore<Move>>(
        &self,
        movelist: &mut T,
    ) {
        if self.is_game_over() {
            // Game is over, so don't generate any moves.
            return;
        }

        let stm = self.color_bb(self.side_to_move);
        let xtm = self.color_bb(!self.side_to_move);
        let gap = self.colored_piece_bb(ColoredPiece::Block);

        // Pieces can only move to unoccupied Squares.
        let allowed = !(stm | xtm | gap);

        for target in moves::singles(stm) & allowed {
            movelist.push(Move::new_single(target));
        }

        for piece in stm {
            // There may be multiple jump moves to a single Square, so they need to be
            // verified (& allowed) and serialized into the movelist immediately.
            let double = moves::double(piece) & allowed;
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

    /// count_moves returns the number of legal moves in the current Position. It
    /// is faster than calling [`Position::generate_moves`] or
    ///  [`Position::generate_moves_into<T>`] and then finding the length.
    fn count_moves<const QUIET: bool, const NOISY: bool>(&self) -> usize {
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
        let mut moves: usize = (moves::singles(stm) & allowed).cardinality();

        for piece in stm {
            // There may be multiple jump moves to a single Square, so they need to be
            // verified (& allowed) and counted into the Position total immediately.
            let double = moves::double(piece) & allowed;
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
    BadSideToMove(#[from] ColorParseError),
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

        let mut rank = Some(Rank::Seventh);
        let mut file = Some(File::A);

        // Iterate over the Ranks in the string spec.
        for rank_data in ranks {
            // Rank pointer ran out, but data carried on.
            if rank.is_none() {
                return Err(PositionParseError::TooManyRanks);
            }

            // Iterate over the Square specs in the Rank spec.
            for data in rank_data.chars() {
                // Check if a jump spec was too big and we landed on an invalid File.
                if file.is_none() {
                    return Err(PositionParseError::JumpTooLong);
                }
                let square = Square::new(file.unwrap(), rank.unwrap());
                match data {
                    'o' | 'O' | 'w' | 'W' => position.insert(square, ColoredPiece::White),
                    'x' | 'X' | 'b' | 'B' => position.insert(square, ColoredPiece::Black),
                    '-' => position.insert(square, ColoredPiece::Block),

                    // Numbers represent jump specs to jump over empty squares.
                    '1'..='8' => {
                        file =
                            File::from_usize(file.unwrap() as usize + data as usize - '1' as usize);
                        if file.is_none() {
                            return Err(PositionParseError::JumpTooLong);
                        }
                    }

                    _ => return Err(PositionParseError::InvalidPieceIdent(data)),
                }

                // On to the next Square spec in the Rank spec.
                file = File::from_usize(file.unwrap() as usize + 1);
            }

            // After rank data runs out, file pointer should be
            // at the last file, i.e, rank is completely filled.
            if let Some(file) = file {
                return Err(PositionParseError::FileDataIncomplete(file));
            }

            // Switch rank pointer and reset file pointer.
            rank = Rank::from_usize((rank.unwrap() as usize).wrapping_sub(1));
            file = Some(File::A);
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
