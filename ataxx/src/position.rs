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

use strum::IntoEnumIterator;

use thiserror::Error;

#[rustfmt::skip]
use crate::{
    BitBoard, Piece, File, Hash, Move,
    MoveList, MoveStore, Rank, Square,
    PieceParseError,
};

/// Position represents the snapshot of an Ataxx Board, the state of the an
/// ataxx game at a single point in time. It also provides all of the methods
/// necessary to manipulate such a snapshot.
#[derive(Copy, Clone)]
pub struct Position {
    /// bitboards stores [BitBoard]s for the piece configuration of each [Piece].
    pub bitboards: [BitBoard; Piece::N],
    /// checksum stores the semi-unique [struct@Hash] of the current Position.
    pub checksum: Hash,
    /// side_to_move stores the [Piece] whose turn to move it currently is.
    pub side_to_move: Piece,
    pub ply_count: u16,
    /// half-move clock stores the number of half-moves since the last irreversible
    /// Move. It is used to adjudicate games using the 50-move/100-ply rule.
    pub half_move_clock: u8,
}

impl Position {
    /// new creates a new Position with the given BitBoards and side to move.
    pub fn new(
        black: BitBoard,
        white: BitBoard,
        block: BitBoard,
        side_to_move: Piece,
        ply_count: u16,
        half_move_clock: u8,
    ) -> Position {
        Position {
            bitboards: [black, white, block],
            checksum: Hash::new(black, white, side_to_move),
            side_to_move,
            ply_count,
            half_move_clock,
        }
    }

    /// put puts the given piece represented by its Piece on the given Square.
    /// ```
    /// use ataxx::*;
    ///
    /// let mut position = Position::new(
    ///     BitBoard::EMPTY,
    ///     BitBoard::EMPTY,
    ///     BitBoard::EMPTY,
    ///     Piece::Black,
    ///     0, 0
    /// );
    /// position.put(Square::A1, Piece::White);
    ///
    /// assert_eq!(position.bitboard(Piece::White), bitboard! {
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     X . . . . . .
    /// });
    /// ```
    pub fn put(&mut self, sq: Square, piece: Piece) {
        debug_assert_ne!(piece, Piece::None);
        self.bitboards[piece as usize].insert(sq);
    }

    /// at returns the Piece of the piece present on the given Square.
    /// ```
    /// use ataxx::*;
    ///
    /// let position = Position::new(
    ///     BitBoard::UNIVERSE,
    ///     BitBoard::EMPTY,
    ///     BitBoard::EMPTY,
    ///     Piece::Black,
    ///     0, 0
    /// );
    /// assert_eq!(position.at(Square::A1), Piece::Black);
    /// ```
    pub const fn at(&self, sq: Square) -> Piece {
        if self.bitboard(Piece::Block).contains(sq) {
            Piece::Block
        } else if self.bitboard(Piece::Black).contains(sq) {
            Piece::Black
        } else if self.bitboard(Piece::White).contains(sq) {
            Piece::White
        } else {
            Piece::None
        }
    }

    /// bitboard returns the BitBoard associated to the piece configuration of the
    /// given Piece. Only the Squares with a piece of the given Piece on them are
    /// contained inside the returned BitBoard.
    /// ```
    /// use ataxx::*;
    ///
    /// let position = Position::new(
    ///     BitBoard::UNIVERSE,
    ///     BitBoard::EMPTY,
    ///     BitBoard::EMPTY,
    ///     Piece::Black,
    ///     0, 0
    /// );
    /// assert_eq!(position.bitboard(Piece::Black), BitBoard::UNIVERSE);
    /// ```
    pub const fn bitboard(&self, piece: Piece) -> BitBoard {
        self.bitboards[piece as usize]
    }
}

impl Position {
    /// is_game_over checks if the game is over, i.e. is a win or a draw.
    /// ```
    /// use ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let black_win = Position::from_str("xxxxxxx/7/7/7/7/7/7 o 0 1").unwrap();
    /// let white_win = Position::from_str("ooooooo/7/7/7/7/7/7 x 0 1").unwrap();
    /// let draw_game = Position::from_str("xxx1ooo/7/7/7/7/7/7 x 100 1").unwrap();
    /// let ongoing = Position::from_str("xxx1ooo/7/7/7/7/7/7 x 0 1").unwrap();
    ///
    /// assert!(black_win.is_game_over());
    /// assert!(white_win.is_game_over());
    /// assert!(draw_game.is_game_over());
    /// assert!(!ongoing.is_game_over());
    /// ```
    pub fn is_game_over(&self) -> bool {
        let black = self.bitboard(Piece::Black);
        let white = self.bitboard(Piece::White);
        let block = self.bitboard(Piece::Block);

        self.half_move_clock >= 100 ||                           // Fifty-move rule
			white | black | block == BitBoard::UNIVERSE ||       // All squares occupied
			white == BitBoard::EMPTY || black == BitBoard::EMPTY // No pieces left
    }

    /// winner returns the Piece which has won the game. It returns [`Piece::None`]
    /// if the game is a draw. If [`Position::is_game_over`] is false, then the
    /// behavior of this function is undefined.
    /// ```
    /// use ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let black_win = Position::from_str("xxxxxxx/7/7/7/7/7/7 o 0 1").unwrap();
    /// let white_win = Position::from_str("ooooooo/7/7/7/7/7/7 x 0 1").unwrap();
    /// let draw_game = Position::from_str("xxx1ooo/7/7/7/7/7/7 x 100 1").unwrap();
    ///
    /// assert_eq!(black_win.winner(), Piece::Black);
    /// assert_eq!(white_win.winner(), Piece::White);
    /// assert_eq!(draw_game.winner(), Piece::None);
    /// ```
    pub fn winner(&self) -> Piece {
        debug_assert!(self.is_game_over());

        if self.half_move_clock >= 100 {
            // Draw by 50 move rule.
            return Piece::None;
        }

        let black = self.bitboard(Piece::Black);
        let white = self.bitboard(Piece::White);
        let block = self.bitboard(Piece::Block);

        if black == BitBoard::EMPTY {
            // Black lost all its pieces, White won.
            return Piece::White;
        } else if white == BitBoard::EMPTY {
            // White lost all its pieces, Black won.
            return Piece::Black;
        }

        debug_assert!(black | white | block == BitBoard::UNIVERSE);

        // All the squares are occupied by pieces. Victory is decided by
        // which Piece has the most number of pieces on the Board.

        let black_n = black.cardinality();
        let white_n = white.cardinality();

        match black_n.cmp(&white_n) {
            cmp::Ordering::Less => Piece::White,
            cmp::Ordering::Greater => Piece::Black,
            // Since there are an odd number of Squares and all of them are filled, there
            // can't be an equal number of black and white pieces on the Board.
            cmp::Ordering::Equal => unreachable!(),
        }
    }
}

impl Position {
    /// after_move returns a new Position which occurs when the given Move is
    /// played on the current Position. Its behavior is undefined if the given
    /// Move is illegal.
    /// ```
    /// use ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let mut pos = Position::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    /// let new_pos = Position::from_str("xx4o/7/7/7/7/7/o5x o 0 1").unwrap();
    ///
    /// let mov = Move::new_single(Square::B7);
    ///
    /// assert_eq!(pos.after_move(mov).checksum, new_pos.checksum);
    /// ```
    pub fn after_move(&self, m: Move) -> Position {
        let stm = self.side_to_move;

        // A pass move is a do nothing move; just change the side to move.
        if m == Move::PASS {
            return Position {
                bitboards: self.bitboards,
                checksum: Hash(!self.checksum.0),
                side_to_move: !self.side_to_move,
                ply_count: self.ply_count + 1,
                half_move_clock: self.half_move_clock + 1,
            };
        }

        let stm_pieces = self.bitboard(stm);
        let xtm_pieces = self.bitboard(!stm);

        let captured = BitBoard::single(m.target()) & xtm_pieces;

        // Move the captured pieces from xtm to stm.
        let new_xtm = xtm_pieces ^ captured;
        let new_stm = stm_pieces ^ captured;

        // Add a stm piece to the target square.
        let mut new_stm = new_stm | BitBoard::from(m.target());

        // Reset half move clock on a singular move.
        let mut half_move_clock = 0;

        // Remove the piece from the source square if the move is non-singular.
        if !m.is_single() {
            new_stm ^= BitBoard::from(m.source());
            // Jump move, so don't reset half move clock.
            half_move_clock = self.half_move_clock + 1;
        };

        let (white, black) = if stm == Piece::White {
            (new_stm, new_xtm)
        } else {
            (new_xtm, new_stm)
        };

        Position {
            bitboards: [black, white, self.bitboard(Piece::Block)],
            checksum: Hash::new(black, white, !stm),
            side_to_move: !stm,
            ply_count: self.ply_count + 1,
            half_move_clock,
        }
    }
}

impl Position {
    /// generate_moves generates the legal moves in the current Position and
    /// returns a [`MoveList`] containing all the moves. It is a wrapper on top of
    /// the more general [`Position::generate_moves_into<T>`].
    /// ```
    /// use ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let position = Position::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    /// let movelist = position.generate_moves();
    ///
    /// // There are 16 possible moves in startpos.
    /// assert_eq!(movelist.len(), 16);
    /// ```
    pub fn generate_moves(&self) -> MoveList {
        let mut movelist = MoveList::new();
        self.generate_moves_into(&mut movelist);
        movelist
    }

    /// generate_moves_into generates all the legal moves in the current Position
    /// and adds them to the given movelist. The type of the movelist must
    /// implement the [`MoveStore`] trait.
    /// ```
    /// use ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let position = Position::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    /// let mut movelist = MoveList::new();
    ///
    /// position.generate_moves_into(&mut movelist);
    ///
    /// // There are 16 possible moves in startpos.
    /// assert_eq!(movelist.len(), 16);
    /// ```
    pub fn generate_moves_into<T: MoveStore>(&self, movelist: &mut T) {
        if self.is_game_over() {
            // Game is over, so don't generate any moves.
            return;
        }

        let stm = self.bitboard(self.side_to_move);
        let xtm = self.bitboard(!self.side_to_move);
        let gap = self.bitboard(Piece::Block);

        // Pieces can only move to unoccupied Squares.
        let allowed = !(stm | xtm | gap);

        let mut single = BitBoard::EMPTY;
        for piece in stm {
            // All single moves to a single Square are equivalent, so a single
            // BitBoard is sufficient to keep track of them all and cast out duplicates.
            // An intersection with the allowed BitBoard is done once at the end.
            single |= BitBoard::single(piece);

            // There may be multiple jump moves to a single Square, so they need to be
            // verified (& allowed) and serialized into the movelist immediately.
            let double = BitBoard::double(piece) & allowed;
            for target in double {
                movelist.push(Move::new(piece, target));
            }
        }

        // Serialize the single moves into the movelist.
        single &= allowed;
        for target in single {
            movelist.push(Move::new_single(target));
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
    /// ```
    /// use ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let position = Position::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    ///
    /// // There are 16 possible moves in startpos.
    /// assert_eq!(position.count_moves(), 16);
    /// ```
    pub fn count_moves(&self) -> usize {
        if self.is_game_over() {
            // Game is over, so don't generate any moves.
            return 0;
        }

        let mut moves: usize = 0;

        let stm = self.bitboard(self.side_to_move);
        let xtm = self.bitboard(!self.side_to_move);
        let gap = self.bitboard(Piece::Block);

        // Pieces can only move to unoccupied Squares.
        let allowed = !(stm | xtm | gap);

        let mut single = BitBoard::EMPTY;
        for piece in stm {
            // All single moves to a single Square are equivalent, so a single
            // BitBoard is sufficient to keep track of them all and cast out duplicates.
            // An intersection with the allowed BitBoard is done once at the end.
            single |= BitBoard::single(piece);

            // There may be multiple jump moves to a single Square, so they need to be
            // verified (& allowed) and counted into the Position total immediately.
            let double = BitBoard::double(piece) & allowed;
            moves += double.cardinality();
        }

        // Count the number single moves in the Position.
        moves += (single & allowed).cardinality();

        // If there are no legal moves possible on the Position and the game isn't
        // over, a pass move is the only move possible to be played.
        if moves == 0 {
            moves += 1;
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
    BadSideToMove(#[from] PieceParseError),
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

        let mut position = Position::new(
            BitBoard::EMPTY,
            BitBoard::EMPTY,
            BitBoard::EMPTY,
            Piece::None,
            0,
            0,
        );

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
                    'o' | 'O' | 'w' | 'W' => position.put(square, Piece::White),
                    'x' | 'X' | 'b' | 'B' => position.put(square, Piece::Black),
                    '-' => position.put(square, Piece::Block),

                    // Numbers represent jump specs to jump over empty squares.
                    '1'..='8' => {
                        file =
                            File::try_from(file.unwrap() as usize + data as usize - '1' as usize);
                        if file.is_err() {
                            return Err(PositionParseError::JumpTooLong);
                        }
                    }

                    _ => return Err(PositionParseError::InvalidPieceIdent(data)),
                }

                // On to the next Square spec in the Rank spec.
                file = File::try_from(file.unwrap() as usize + 1);
            }

            // After rank data runs out, file pointer should be
            // at the last file, i.e, rank is completely filled.
            if let Ok(file) = file {
                return Err(PositionParseError::FileDataIncomplete(file));
            }

            // Switch rank pointer and reset file pointer.
            rank = Rank::try_from((rank.unwrap() as usize).wrapping_sub(1));
            file = Ok(File::A);
        }

        position.side_to_move = Piece::from_str(stm)?;
        position.half_move_clock = hmc.parse::<u8>()?;
        position.ply_count = fmc.parse::<u16>()? * 2 - 1;
        if position.side_to_move == Piece::Black {
            position.ply_count -= 1;
        }

        // Calculate the Hash value for the Position.
        position.checksum = Hash::new(
            position.bitboard(Piece::Black),
            position.bitboard(Piece::White),
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
                string_rep += &format!("{} ", board.at(square));
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
