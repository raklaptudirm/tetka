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

use std::fmt;
use std::str::FromStr;
use crate::ataxx::hash::Hash;
use super::{BitBoard, Move, Color, Square, File, Rank, FEN, MoveList};
use strum::IntoEnumIterator;

pub struct Board {
    history: [Position; Board::MAX_PLY],
    current: usize,
}

impl Board {
    // MAX_PLY is the maximum number of plys than can be in a game.
    const MAX_PLY: usize = 1024;

    const fn current_pos(&self) -> &Position {
        &self.history[self.current]
    }

    // Position returns a copy of the current Position on the Board.
    pub fn position(&self) -> Position { self.history[self.current] }

    // side_to_move returns the current Color to move on the Board.
    pub fn side_to_move(&self) -> Color {
        self.current_pos().side_to_move
    }

    // Checksum returns a semi-unique Hash to identify the Position by.
    pub fn checksum(&self) -> Hash {
        self.current_pos().checksum
    }

    ////////////////////////////////////////////
    // Reimplementation of Position's methods //
    ////////////////////////////////////////////

    pub const fn bitboard(&self, color: Color) -> BitBoard {
        self.current_pos().bitboard(color)
    }

    pub const fn at(&self, sq: Square) -> Color {
        self.current_pos().at(sq)
    }
}

impl Board {
    pub fn make_move(&mut self, m: Move) {
        self.history[self.current + 1] = self.current_pos().after_move(m);
        self.current += 1;
    }

    pub fn undo_move(&mut self) {
        self.current -= 1;
    }
}

impl From<&FEN> for Board {
    fn from(fen: &FEN) -> Self {
        println!("{}", fen);
        let mut board = Board {
            history: [Position::new(BitBoard::EMPTY, BitBoard::EMPTY, Color::None); Board::MAX_PLY],
            current: 0,
        };

        board.history[0] = fen.position;
        board
    }
}

impl Board {
    pub fn generate_moves(&self) -> MoveList {
        self.current_pos().generate_moves()
    }

    pub fn count_moves(&self) -> usize {
        self.current_pos().count_moves()
    }
}

#[derive(Copy, Clone)]
pub struct Position {
    pub bitboards: [BitBoard; Color::N],
    pub checksum: Hash,
    pub side_to_move: Color,
}

impl Position {
    pub fn new(white: BitBoard, black: BitBoard, stm: Color) -> Position {
        Position {
            bitboards: [white, black],
            checksum: Hash::new(white.0, black.0).perspective(stm),
            side_to_move: stm,
        }
    }

    pub const fn bitboard(&self, color: Color) -> BitBoard {
        self.bitboards[color as usize]
    }

    pub fn put(&mut self, sq: Square, color: Color) {
        match color {
            Color::White => self.bitboards[Color::White as usize].insert(sq),
            Color::Black => self.bitboards[Color::Black as usize].insert(sq),
            Color::None => panic!(""),
        };
    }

    pub const fn at(&self, sq: Square) -> Color {
        if self.bitboard(Color::White).contains(sq) {
            Color::White
        } else if self.bitboard(Color::Black).contains(sq) {
            Color::Black
        } else {
            Color::None
        }
    }

    pub fn is_game_over(&self) -> bool {
        let white = self.bitboard(Color::White);
        let black = self.bitboard(Color::Black);
        white | black == BitBoard::UNIVERSE || white == BitBoard::EMPTY || black == BitBoard::EMPTY
    }

    pub fn winner(&self) -> Color {
        let white = self.bitboard(Color::White);
        let black = self.bitboard(Color::Black);

        if white == BitBoard::EMPTY {
            return Color::Black;
        } else if black == BitBoard::EMPTY {
            return Color::White;
        } else if white | black != BitBoard::UNIVERSE {
            return Color::None;
        }

        let white_n = white.cardinality();
        let black_n = black.cardinality();

        if white_n > black_n {
            Color::White
        } else if black_n > white_n {
            Color::Black
        } else {
            Color::None
        }
    }
}

impl Position {
    pub fn after_move(&self, m: Move) -> Position {
        let stm = self.side_to_move;

        let stm_pieces = self.bitboard(stm);
        let xtm_pieces = self.bitboard(!stm);

        let captured = BitBoard::SINGLES[m.target() as usize] & xtm_pieces;

        // Move the captured pieces from xtm to stm.
        let new_xtm = xtm_pieces ^ captured;
        let new_stm = stm_pieces ^ captured;

        // Add a stm piece to the target square.
        let mut new_stm = new_stm | BitBoard::from(m.target());

        // Remove the piece from the source square if the move is non-singular.
        if !m.is_single() {
            new_stm ^= BitBoard::from(m.source())
        };

        if stm == Color::White {
            Position::new(new_stm, new_xtm, !stm)
        } else {
            Position::new(new_xtm, new_stm, !stm)
        }
    }
}

impl Position {
    pub fn generate_moves(&self) -> MoveList {
        let mut movelist = MoveList::new();

        let stm = self.bitboard(self.side_to_move);
        let xtm = self.bitboard(!self.side_to_move);

        let allowed = !(stm | xtm);

        let mut single = BitBoard::EMPTY;
        for piece in stm {
            single |= BitBoard::SINGLES[piece as usize];

            let double = BitBoard::DOUBLES[piece as usize] & allowed;
            for target in double {
                movelist.push(Move::new(piece, target));
            }
        }

        single &= allowed;
        for target in single {
            movelist.push(Move::new_single(target));
        }

        movelist
    }

    pub fn count_moves(&self) -> usize {
        let mut moves: usize = 0;

        let stm = self.bitboard(self.side_to_move);
        let xtm = self.bitboard(!self.side_to_move);

        let allowed = !(stm | xtm);

        let mut single = BitBoard::EMPTY;
        for piece in stm {
            single |= BitBoard::SINGLES[piece as usize];

            let double = BitBoard::DOUBLES[piece as usize] & allowed;
            moves += double.cardinality() as usize;
        }

        moves + (single & allowed).cardinality() as usize
    }
}

#[derive(Debug)]
pub enum PositionParseErr {
    JumpTooLong,
    InvalidColorIdent,
    FileDataIncomplete,
    TooManyFields,
}

impl FromStr for Position {
    type Err = PositionParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut position = Position::new(BitBoard::EMPTY, BitBoard::EMPTY, Color::None);

        let ranks: Vec<&str> = s.split('/').collect();

        let mut rank = Ok(Rank::Seventh);
        let mut file = Ok(File::A);
        for rank_data in ranks {
            // Rank pointer ran out, but data carried on.
            if rank.is_err() {
                return Err(PositionParseErr::TooManyFields);
            }

            for data in rank_data.chars() {
                if file.is_err() {
                    return Err(PositionParseErr::JumpTooLong);
                }
                let square = Square::new(file.unwrap(), rank.unwrap());
                match data {
                    'O' | 'o' | 'w' => position.put(square, Color::White),
                    'X' | 'x' | 'b' => position.put(square, Color::Black),

                    '1'..='8' => {
                        file = File::try_from(file.unwrap() as usize + data as usize - '1' as usize);
                        if file.is_err() {
                            return Err(PositionParseErr::JumpTooLong);
                        }
                    }

                    _ => return Err(PositionParseErr::InvalidColorIdent),
                }

                file = File::try_from(file.unwrap() as usize + 1);
            }

            // After rank data runs out, file pointer should be
            // at the last file, i.e, rank is completely filled.
            if file.is_ok() {
                return Err(PositionParseErr::FileDataIncomplete);
            }

            // Switch rank pointer and reset file pointer.
            rank = Rank::try_from((rank.unwrap() as usize).wrapping_sub(1));
            file = Ok(File::A);
        }

        position.checksum = Hash::new(
            position.bitboard(Color::White).0,
            position.bitboard(Color::Black).0,
        );

        Ok(position)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board = self;
        let mut string_rep = String::from(" ");

        for rank in Rank::iter().rev() {
            for file in File::iter() {
                let square = Square::new(file, rank);
                string_rep += &format!("{} ", board.at(square));
            }

            string_rep += &format!(" {} \n ", rank);
        }

        string_rep += "a b c d e f g\n";

        write!(f, "{}\n", string_rep).unwrap();
        write!(f, "Side To Move: {}\n", self.side_to_move)
    }
}
