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

use crate::interface;
use crate::interface::ColoredPieceType;
use crate::interface::PiecePlacementParseError;
use crate::interface::PositionType;
use crate::interface::TypeParseError;
use crate::interface::{BitBoardType, Hash, RepresentableType, SquareType};

use thiserror::Error;

#[rustfmt::skip]
use super::{
    BitBoard, ColoredPiece, File, Move,
    Rank, Square, Color, Piece
};
use crate::interface::MoveStore;

/// Position represents the snapshot of an Ataxx Board, the state of the an
/// ataxx game at a single point in time. It also provides all of the methods
/// necessary to manipulate such a snapshot.
#[derive(Copy, Clone)]
pub struct Position {
    pub pawns: [Square; Color::N],
    pub tiles: BitBoard,
    /// checksum stores the semi-unique [struct@Hash] of the current Position.
    pub checksum: Hash,
    /// side_to_move stores the piece whose turn to move it currently is.
    pub side_to_move: Color,
    pub ply_count: u16,
}

impl PositionType for Position {
    type BitBoard = BitBoard;
    type ColoredPiece = ColoredPiece;
    type Move = Move;

    fn insert(&mut self, sq: Square, piece: ColoredPiece) {
        match piece.piece() {
            Piece::Pawn => self.pawns[piece as usize] = sq,
            Piece::Tile => self.tiles.insert(sq),
        }
    }

    fn remove(&mut self, sq: Square) -> Option<ColoredPiece> {
        if self.pawns[Color::White as usize] == sq {
            Some(ColoredPiece::WhitePawn)
        } else if self.pawns[Color::Black as usize] == sq {
            Some(ColoredPiece::BlackPawn)
        } else if self.tiles.contains(sq) {
            self.tiles ^= BitBoard::from(sq);
            Some(ColoredPiece::Tile)
        } else {
            None
        }
    }

    fn at(&self, sq: Square) -> Option<ColoredPiece> {
        ColoredPiece::iter()
            .find(|piece| self.colored_piece_bb(*piece).contains(sq))
    }

    fn piece_bb(&self, piece: Piece) -> BitBoard {
        match piece {
            Piece::Pawn => BitBoard::from(self.pawns[0]) | self.pawns[1],
            Piece::Tile => self.tiles,
        }
    }

    fn color_bb(&self, color: Color) -> BitBoard {
        BitBoard::from(self.pawns[color as usize])
    }

    fn colored_piece_bb(&self, piece: ColoredPiece) -> BitBoard {
        match piece.piece() {
            Piece::Pawn => BitBoard::from(self.pawns[piece as usize]),
            Piece::Tile => self.tiles,
        }
    }

    fn hash(&self) -> Hash {
        self.checksum
    }

    fn is_game_over(&self) -> bool {
        let black = self.colored_piece_bb(ColoredPiece::WhitePawn);
        let white = self.colored_piece_bb(ColoredPiece::BlackPawn);
        let block = self.colored_piece_bb(ColoredPiece::Tile);

        white | black | block == BitBoard::UNIVERSE ||       // All squares occupied
			white == BitBoard::EMPTY || black == BitBoard::EMPTY // No pieces left
    }

    fn winner(&self) -> Option<Color> {
        let black = self.colored_piece_bb(ColoredPiece::WhitePawn);
        let white = self.colored_piece_bb(ColoredPiece::BlackPawn);
        let block = self.colored_piece_bb(ColoredPiece::Tile);

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

        let black_n = black.len();
        let white_n = white.len();

        match black_n.cmp(&white_n) {
            cmp::Ordering::Less => Some(Color::White),
            cmp::Ordering::Greater => Some(Color::Black),
            // Though there can't be an equal number of black and white pieces
            // on an empty ataxx board, it is possible with an odd number of
            // blocker pieces.
            cmp::Ordering::Equal => None,
        }
    }

    fn after_move<const UPDATE_HASH: bool>(&self, m: Move) -> Position {
        let stm = self.side_to_move;

        macro_rules! update_hash {
            ($e:expr) => {
                if UPDATE_HASH {
                    $e
                } else {
                    Default::default()
                }
            };
        }

        let xtm_pawn = self.pawns[!stm as usize];
        let stm_pawn = m.pawn();
        let tiles = self.colored_piece_bb(ColoredPiece::Tile)
            ^ BitBoard::from(m.tile());

        let (white, black) = if stm == Color::White {
            (stm_pawn, xtm_pawn)
        } else {
            (xtm_pawn, stm_pawn)
        };

        Position {
            pawns: [white, black],
            tiles,
            checksum: update_hash!(Self::get_hash(white, black, tiles, !stm)),
            side_to_move: !stm,
            ply_count: self.ply_count + 1,
        }
    }

    fn generate_moves_into<
        const ALLOW_ILLEGAL: bool,
        const QUIET: bool,
        const NOISY: bool,
        T: MoveStore<Move>,
    >(
        &self,
        movelist: &mut T,
    ) {
        let stm = self.color_bb(self.side_to_move);
        let xtm = self.color_bb(!self.side_to_move);

        let tiles = self.colored_piece_bb(ColoredPiece::Tile);

        // Pieces can only move to unoccupied Squares.
        let allowed = tiles - xtm;

        for target in BitBoard::singles(stm) & allowed {
            for tile in allowed ^ BitBoard::from(target) {
                movelist.push(Move::new(target, tile));
            }
        }
    }

    fn count_moves<const QUIET: bool, const NOISY: bool>(&self) -> usize {
        let stm = self.color_bb(self.side_to_move);
        let xtm = self.color_bb(!self.side_to_move);

        let tiles = self.colored_piece_bb(ColoredPiece::Tile);

        // Pieces can only move to unoccupied Squares.
        let allowed = tiles - xtm;

        (BitBoard::singles(stm) & allowed).count() * (allowed.count() - 1)
    }
}

impl Position {
    fn get_hash(
        white: Square,
        black: Square,
        tiles: BitBoard,
        stm: Color,
    ) -> Hash {
        let a = white as u64 * black as u64;
        let b = tiles.into();

        // Currently, an 2^-63-almost delta universal hash function, based on
        // https://eprint.iacr.org/2011/116.pdf by Long Hoang Nguyen and Andrew
        // William Roscoe is used to create the Hash. This may change in the future.

        // 3 64-bit integer constants used in the hash function.
        const X: u64 = 6364136223846793005;
        const Y: u64 = 1442695040888963407;
        const Z: u64 = 2305843009213693951;

        // xa + yb + floor(ya/2^64) + floor(zb/2^64)
        // floor(pq/2^64) is essentially getting the top 64 bits of p*q.
        let part_1 = X.wrapping_mul(a); // xa
        let part_2 = Y.wrapping_mul(b); // yb
        let part_3 = (Y as u128 * a as u128) >> 64; // floor(ya/2^64) = ya >> 64
        let part_4 = (Z as u128 * b as u128) >> 64; // floor(zb/2^64) = zb >> 64

        // add the parts together and return the resultant hash.
        let hash = part_1
            .wrapping_add(part_2)
            .wrapping_add(part_3 as u64)
            .wrapping_add(part_4 as u64);

        // The Hash is bitwise complemented if the given side to move is Black.
        // Therefore, if two Positions only differ in side to move,
        // `a.Hash == !b.Hash`.
        if stm == Color::Black {
            Hash::new(!hash)
        } else {
            Hash::new(hash)
        }
    }
}

/// PositionParseErr represents an error encountered while parsing
/// the given FEN position field into a valid Position.
#[derive(Error, Debug)]
pub enum PositionParseError {
    #[error("expected 3 fields, found {0}")]
    TooManyFields(usize),

    #[error("parsing piece placement: {0}")]
    BadPiecePlacement(#[from] PiecePlacementParseError),

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

        if parts.len() != 3 {
            return Err(PositionParseError::TooManyFields(parts.len()));
        }

        let pos = parts[0];
        let stm = parts[1];
        let fmc = parts[2];

        let mut position = Position {
            pawns: [Square::A1, Square::A1],
            tiles: BitBoard::EMPTY,
            checksum: Default::default(),
            side_to_move: Color::Black,
            ply_count: 0,
        };

        interface::parse_piece_placement(&mut position, pos)?;

        position.side_to_move = Color::from_str(stm)?;
        position.ply_count = fmc.parse::<u16>()? * 2 - 1;
        if position.side_to_move == Color::Black {
            position.ply_count -= 1;
        }

        // Calculate the Hash value for the Position.
        unsafe {
            position.checksum = Self::get_hash(
                position
                    .colored_piece_bb(ColoredPiece::WhitePawn)
                    .next()
                    .unwrap_unchecked(),
                position
                    .colored_piece_bb(ColoredPiece::BlackPawn)
                    .next()
                    .unwrap_unchecked(),
                position.colored_piece_bb(ColoredPiece::Tile),
                position.side_to_move,
            );
        }

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
