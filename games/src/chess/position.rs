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

use strum::IntoEnumIterator;

use crate::interface;
use crate::interface::ColoredPieceType;
use crate::interface::PiecePlacementParseError;
use crate::interface::PositionType;
use crate::interface::TypeParseError;
use crate::interface::{BitBoardType, Hash, RepresentableType, SquareType};

use thiserror::Error;

#[rustfmt::skip]
use crate::chess::{
    BitBoard, ColoredPiece, File, Move,
    Rank, Square, Color, Piece, castling
};
use crate::interface::MoveStore;

/// Position represents the snapshot of an Ataxx Board, the state of the an
/// ataxx game at a single point in time. It also provides all of the methods
/// necessary to manipulate such a snapshot.
#[derive(Clone)]
pub struct Position {
    // BitBoard board representation.
    pub color_bbs: [BitBoard; Color::N],
    pub piece_bbs: [BitBoard; Piece::N],

    // Position metadata.
    side_to_move: Color,
    ply_count: u16,
    half_move_clock: u8,

    #[allow(dead_code)]
    en_passant_target: Option<Square>,

    // Game metadata.
    #[allow(dead_code)]
    is_fischer_random: bool,
    #[allow(dead_code)]
    castling_square_info: castling::Info,
    checksum: Hash,
}

impl PositionType for Position {
    type BitBoard = BitBoard;
    type ColoredPiece = ColoredPiece;
    type Move = Move;

    fn insert(&mut self, sq: Square, piece: ColoredPiece) {
        self.piece_bbs[piece.piece() as usize].insert(sq);
        self.color_bbs[piece.color() as usize].insert(sq);
    }

    fn remove(&mut self, sq: Square) -> Option<ColoredPiece> {
        match self.at(sq) {
            Some(piece) => {
                self.piece_bbs[piece.piece() as usize].remove(sq);
                self.color_bbs[piece.color() as usize].remove(sq);
                Some(piece)
            }
            None => None,
        }
    }

    fn at(&self, sq: Square) -> Option<ColoredPiece> {
        ColoredPiece::iter()
            .find(|piece| self.colored_piece_bb(*piece).contains(sq))
    }

    fn piece_bb(&self, piece: Piece) -> BitBoard {
        self.piece_bbs[piece as usize]
    }

    fn color_bb(&self, color: Color) -> BitBoard {
        self.color_bbs[color as usize]
    }

    fn colored_piece_bb(&self, piece: ColoredPiece) -> BitBoard {
        self.piece_bb(piece.piece()) & self.color_bb(piece.color())
    }

    fn hash(&self) -> Hash {
        self.checksum
    }

    fn is_game_over(&self) -> bool {
        false
    }

    fn winner(&self) -> Option<Color> {
        None
    }

    fn after_move<const UPDATE_HASH: bool>(&self, _m: Move) -> Position {
        self.clone()
    }

    fn generate_moves_into<
        const ALLOW_ILLEGAL: bool,
        const QUIET: bool,
        const NOISY: bool,
        T: MoveStore<Move>,
    >(
        &self,
        _movelist: &mut T,
    ) {
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

        if parts.len() != 4 {
            return Err(PositionParseError::TooManyFields(parts.len()));
        }

        let pos = parts[0];
        let stm = parts[1];
        let hmc = parts[2];
        let fmc = parts[3];

        let mut position = Position {
            color_bbs: [BitBoard::EMPTY; Color::N],
            piece_bbs: [BitBoard::EMPTY; Piece::N],
            checksum: Default::default(),
            side_to_move: Color::Black,
            ply_count: 0,
            half_move_clock: 0,
            en_passant_target: None,
            is_fischer_random: false,
            castling_square_info: castling::Info::from_squares(
                Square::E1,
                File::H,
                File::A,
                Square::E8,
                File::H,
                File::A,
            ),
        };

        interface::parse_piece_placement(&mut position, pos)?;

        position.side_to_move = Color::from_str(stm)?;
        position.half_move_clock = hmc.parse::<u8>()?;
        position.ply_count = fmc.parse::<u16>()? * 2 - 1;
        if position.side_to_move == Color::Black {
            position.ply_count -= 1;
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
