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

use std::{fmt, num::ParseIntError, str::FromStr};

use super::{
    castling::{self, CastlingRightsParseError, Dimension, Rights, Side},
    movegen, BitBoard, Color, ColoredPiece, File, Move, MoveFlag,
    MoveParseError, Piece, Rank, Square,
};
use crate::interface::{
    self, parse::PiecePlacementParseError, ColoredPieceType, Hash, MoveStore,
    PositionType, RepresentableType, SetType, SquareType, TypeParseError,
};

use strum::IntoEnumIterator;
use thiserror::Error;

/// Position represents the snapshot of an Ataxx Board, the state of the an
/// ataxx game at a single point in time. It also provides all of the methods
/// necessary to manipulate such a snapshot.
#[derive(Clone)]
pub struct Position {
    // BitBoard board representation.
    color_bbs: [BitBoard; Color::N],
    piece_bbs: [BitBoard; Piece::N],

    // Position metadata.
    side_to_move: Color,
    ply_count: u16,
    half_move_clock: u8,

    en_passant_target: Option<Square>,

    // Game metadata.
    is_fischer_random: bool,
    castling: castling::Info,
    checksum: Hash,
}

impl PositionType for Position {
    type Square = Square;
    type ColoredPiece = ColoredPiece;
    type Move = Move;
    type MoveParseError = MoveParseError;

    const STARTPOS: &str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    fn parse_move(
        &self,
        move_str: &str,
    ) -> Result<Self::Move, Self::MoveParseError> {
        if move_str.len() < 4 || move_str.len() > 5 {
            return Err(MoveParseError::BadLength(move_str.len()));
        }

        let source = Square::from_str(&move_str[0..2])?;
        let target = Square::from_str(&move_str[2..4])?;

        let source_piece = match self.at(source) {
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
        } else if is_pawn && self.en_passant_target == Some(target) {
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

    fn insert(&mut self, sq: Square, piece: ColoredPiece) {
        self.piece_bbs[piece.piece()].insert(sq);
        self.color_bbs[piece.color()].insert(sq);
    }

    fn remove(&mut self, sq: Square) -> Option<ColoredPiece> {
        match self.at(sq) {
            Some(piece) => {
                self.piece_bbs[piece.piece()].remove(sq);
                self.color_bbs[piece.color()].remove(sq);
                Some(piece)
            }
            None => None,
        }
    }

    fn at(&self, sq: Square) -> Option<ColoredPiece> {
        ColoredPiece::iter()
            .find(|piece| self.colored_piece_bb(*piece).contains(sq))
    }

    fn side_to_move(&self) -> interface::Color<Self> {
        self.side_to_move
    }

    fn half_move_clock(&self) -> usize {
        self.half_move_clock as usize
    }

    fn ply_count(&self) -> usize {
        self.ply_count as usize
    }

    fn hash(&self) -> Hash {
        self.checksum
    }

    fn is_game_over(&self) -> bool {
        false
    }

    fn winner(&self) -> Option<Option<Color>> {
        None
    }

    fn after_move<const UPDATE_HASH: bool>(&self, m: Move) -> Position {
        let mut board = self.clone();

        let source_pc = board.remove(m.source());
        let target_pc = board.remove(m.target());

        let source_pc = unsafe { source_pc.unwrap_unchecked() };

        board.castling.rights -= board.castling.get_updates(m.source())
            | board.castling.get_updates(m.target());

        board.en_passant_target = None;

        match m.flag() {
            MoveFlag::Normal => board.insert(m.target(), source_pc),
            MoveFlag::NPromotion
            | MoveFlag::BPromotion
            | MoveFlag::RPromotion
            | MoveFlag::QPromotion => board.insert(
                m.target(),
                ColoredPiece::new(
                    unsafe { m.flag().promoted_piece() },
                    board.side_to_move,
                ),
            ),
            MoveFlag::EnPassant => {
                board.insert(m.target(), source_pc);
                board.remove(unsafe {
                    m.target().down(board.side_to_move).unwrap_unchecked()
                });
            }
            MoveFlag::DoublePush => {
                board.insert(m.target(), source_pc);
                board.en_passant_target = Some(unsafe {
                    m.target().down(board.side_to_move).unwrap_unchecked()
                });
            }
            MoveFlag::CastleHSide | MoveFlag::CastleASide => {
                let (king, rook) = Dimension::from(
                    board.side_to_move,
                    if m.flag() == MoveFlag::CastleHSide {
                        Side::H
                    } else {
                        Side::A
                    },
                )
                .get_targets();
                board.insert(
                    king,
                    ColoredPiece::new(Piece::King, board.side_to_move),
                );
                board.insert(
                    rook,
                    ColoredPiece::new(Piece::Rook, board.side_to_move),
                );
            }
        }

        board.half_move_clock += 1;
        if source_pc.piece() == Piece::Pawn || target_pc.is_some() {
            board.half_move_clock = 0;
        }
        board.side_to_move = !board.side_to_move;

        board
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
        let info = movegen::MoveGenerationInfo::new(self);
        info.generate_moves_into(movelist);
    }
}

impl Position {
    pub fn piece_bb(&self, piece: Piece) -> BitBoard {
        self.piece_bbs[piece]
    }

    pub fn color_bb(&self, color: Color) -> BitBoard {
        self.color_bbs[color]
    }

    pub fn colored_piece_bb(&self, piece: ColoredPiece) -> BitBoard {
        self.piece_bb(piece.piece()) & self.color_bb(piece.color())
    }
    pub fn en_passant_target(&self) -> Option<Square> {
        self.en_passant_target
    }

    pub fn castling(&self) -> &castling::Info {
        &self.castling
    }

    pub fn is_frc(&self) -> bool {
        self.is_fischer_random
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

    #[error("position missing a king")]
    MissingKing,

    #[error("parsing side to move: {0}")]
    BadSideToMove(#[from] TypeParseError),
    #[error("parsing castling rights: {0}")]
    BadCastlingRights(#[from] CastlingRightsParseError),
    #[error("parsing half-move clock: {0}")]
    BadHalfMoveClock(#[from] ParseIntError),
}

// FromStr implements parsing of the position field in a FEN.
impl FromStr for Position {
    type Err = PositionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect::<Vec<&str>>();

        if parts.len() != 6 {
            return Err(PositionParseError::TooManyFields(parts.len()));
        }

        let pos = parts[0];
        let stm = parts[1];
        let rig = parts[2];
        let ept = parts[3];
        let hmc = parts[4];
        let fmc = parts[5];

        let mut position = Position {
            color_bbs: [BitBoard::EMPTY; Color::N],
            piece_bbs: [BitBoard::EMPTY; Piece::N],
            checksum: Default::default(),
            side_to_move: Color::Black,
            ply_count: 0,
            half_move_clock: 0,
            en_passant_target: None,
            is_fischer_random: false,
            castling: castling::Info::from_squares(
                Square::E1,
                File::H,
                File::A,
                Square::E8,
                File::H,
                File::A,
                Rights::new(),
            ),
        };

        interface::parse::piece_placement(&mut position, pos)?;

        let kings = position.piece_bb(Piece::King);
        let white_king = (kings & position.color_bb(Color::White)).next();
        let black_king = (kings & position.color_bb(Color::Black)).next();

        if let (Some(white_king), Some(black_king)) = (white_king, black_king) {
            position.castling =
                castling::Info::from_str(rig, white_king, black_king)?;
        } else {
            return Err(PositionParseError::MissingKing);
        }

        position.side_to_move = Color::from_str(stm)?;
        position.en_passant_target = if ept == "-" {
            None
        } else {
            Some(Square::from_str(ept)?)
        };
        position.half_move_clock = hmc.parse::<u8>()?;
        position.ply_count =
            interface::parse::ply_count(fmc, position.side_to_move)?;

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
        string_rep += "\n a b c d e f g h\n";

        writeln!(f, "{}", string_rep).unwrap();
        writeln!(f, "Side To Move: {}", self.side_to_move)
    }
}
