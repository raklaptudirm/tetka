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

use std::str::FromStr;

use crate::interface::{
    representable_type, set_type, RepresentableType, SetType, SquareType,
    TypeParseError,
};

use thiserror::Error;

use super::{BitBoard, Color, File, Rank, Square};

set_type!(Rights<Dimension>: u8);

representable_type!(
    enum Dimension: u8 {
        WhiteH "H", WhiteA "A",
        BlackH "h", BlackA "a",
    }
);

impl Dimension {
    pub const N: usize = 4;

    pub fn from(color: Color, side: Side) -> Dimension {
        unsafe {
            Dimension::unsafe_from(color as usize * Color::N + side as usize)
        }
    }

    pub const fn color(self) -> Color {
        match self {
            Dimension::WhiteH | Dimension::WhiteA => Color::White,
            Dimension::BlackH | Dimension::BlackA => Color::Black,
        }
    }

    pub const fn side(self) -> Side {
        match self {
            Dimension::WhiteH | Dimension::BlackH => Side::H,
            Dimension::WhiteA | Dimension::BlackA => Side::A,
        }
    }

    pub fn from_sqs(king_sq: Square, rook_sq: Square) -> Dimension {
        let color: Color = if king_sq.rank() == Rank::First {
            Color::White
        } else {
            Color::Black
        };

        Dimension::from(color, Side::from_sqs(king_sq, rook_sq))
    }

    pub fn get_targets(self) -> (Square, Square) {
        match self {
            Dimension::WhiteH => (Square::G1, Square::F1),
            Dimension::WhiteA => (Square::C1, Square::D1),
            Dimension::BlackH => (Square::G8, Square::F8),
            Dimension::BlackA => (Square::C8, Square::D8),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Side {
    H, A,
}

impl Side {
    pub fn from_sqs(king_sq: Square, rook_sq: Square) -> Side {
        if (king_sq as u8) < rook_sq as u8 {
            Side::H
        } else {
            Side::A
        }
    }
}

#[derive(Clone)]
pub struct Info {
    pub rights: Rights,
    rooks: [Square; Dimension::N],
    attacks_mask: [BitBoard; Dimension::N],
    blocker_mask: [BitBoard; Dimension::N],
    rights_masks: [Rights; Square::N],
}

mod ends {
    use super::Square;

    pub const WHITE_KING_H: Square = Square::G1;
    pub const WHITE_KING_A: Square = Square::C1;
    pub const BLACK_KING_H: Square = Square::G8;
    pub const BLACK_KING_A: Square = Square::C8;

    pub const WHITE_ROOK_H: Square = Square::F1;
    pub const WHITE_ROOK_A: Square = Square::D1;
    pub const BLACK_ROOK_H: Square = Square::F8;
    pub const BLACK_ROOK_A: Square = Square::D8;
}

#[derive(Error, Debug)]
pub enum CastlingRightsParseError {
    #[error("error parsing file string \"{0}\"")]
    FileParseError(#[from] TypeParseError),
    #[error("found invalid castling rights \"{0}\"")]
    Invalid(String),
}

impl Info {
    pub fn from_str(
        s: &str,
        white_king: Square,
        black_king: Square,
    ) -> Result<Self, CastlingRightsParseError> {
        if s == "-" {
            return Ok(Info::from_squares(
                Square::E1,
                File::H,
                File::A,
                Square::E8,
                File::H,
                File::A,
                Rights::new(),
            ));
        }

        if s.is_empty() || s.len() > 4 {
            return Err(CastlingRightsParseError::Invalid(s.to_string()));
        }

        let frc = !matches!(
            unsafe { s.chars().next().unwrap_unchecked() },
            'K' | 'Q' | 'k' | 'q'
        );

        let mut rights = Rights::new();

        let mut white_h = File::H;
        let mut white_a = File::A;
        let mut black_h = File::H;
        let mut black_a = File::A;

        for right in s.chars() {
            if frc {
                if right.is_uppercase() {
                    let file =
                        File::from_str(&right.to_lowercase().to_string())?;
                    if file as usize > white_king.file() as usize {
                        white_h = file;
                        rights = rights | Dimension::WhiteH;
                    } else {
                        white_a = file;
                        rights = rights | Dimension::WhiteA;
                    }
                } else {
                    let file = File::from_str(&right.to_string())?;
                    if file as usize > black_king.file() as usize {
                        black_h = file;
                        rights = rights | Dimension::BlackH;
                    } else {
                        black_a = file;
                        rights = rights | Dimension::BlackA;
                    }
                }
            } else {
                match right {
                    'K' => rights = rights | Dimension::WhiteH,
                    'Q' => rights = rights | Dimension::WhiteA,
                    'k' => rights = rights | Dimension::BlackH,
                    'q' => rights = rights | Dimension::BlackA,
                    _ => {
                        return Err(CastlingRightsParseError::Invalid(
                            right.to_string(),
                        ))
                    }
                }
            }
        }

        Ok(Info::from_squares(
            white_king, white_h, white_a, black_king, black_h, black_a, rights,
        ))
    }

    #[rustfmt::skip]
    pub fn from_squares(
        w_king: Square, w_rook_h: File, w_rook_a: File,
        b_king: Square, b_rook_h: File, b_rook_a: File,
        rights: Rights,
    ) -> Info {
        let mut info = Info {
            rights,
            rooks: [Square::A1; Dimension::N],
            attacks_mask: [BitBoard::EMPTY; Dimension::N],
            blocker_mask: [BitBoard::EMPTY; Dimension::N],
            rights_masks: [Rights::new(); Square::N],
        };

        // Get the bit offsets/indexes of each side-color.
        let wh = Dimension::WhiteH as usize;
        let wa = Dimension::WhiteA as usize;
        let bh = Dimension::BlackH as usize;
        let ba = Dimension::BlackA as usize;

        // Initialize the rook square table.
        info.rooks[wh] = Square::new(w_rook_h, Rank::First);
        info.rooks[wa] = Square::new(w_rook_a, Rank::First);
        info.rooks[bh] = Square::new(b_rook_h, Rank::Eighth);
        info.rooks[ba] = Square::new(b_rook_a, Rank::Eighth);

        // Initialize the castling path table.
        info.blocker_mask[wh] = blocker_mask(w_king, info.rooks[wh], ends::WHITE_KING_H, ends::WHITE_ROOK_H);
        info.blocker_mask[wa] = blocker_mask(w_king, info.rooks[wa], ends::WHITE_KING_A, ends::WHITE_ROOK_A);
        info.blocker_mask[bh] = blocker_mask(b_king, info.rooks[bh], ends::BLACK_KING_H, ends::BLACK_ROOK_H);
        info.blocker_mask[ba] = blocker_mask(b_king, info.rooks[ba], ends::BLACK_KING_A, ends::BLACK_ROOK_A);

        info.attacks_mask[wh] = BitBoard::between2(w_king, ends::WHITE_KING_H);
        info.attacks_mask[wa] = BitBoard::between2(w_king, ends::WHITE_KING_A);
        info.attacks_mask[bh] = BitBoard::between2(b_king, ends::BLACK_KING_H);
        info.attacks_mask[ba] = BitBoard::between2(b_king, ends::BLACK_KING_A);

        fn blocker_mask(king: Square, rook: Square, king_end: Square, rook_end: Square) -> BitBoard {
            (BitBoard::between2(king, king_end) | BitBoard::between2(rook, rook_end)) - (BitBoard::from(king) | BitBoard::from(rook))
        }

        // Initialize the rights update for the king's squares.
        info.rights_masks[w_king as usize] = Rights::new() | Dimension::WhiteH | Dimension::WhiteA;
        info.rights_masks[b_king as usize] = Rights::new() | Dimension::BlackH | Dimension::BlackA;

        // Initialize the rights update for the rook's squares.
        info.rights_masks[info.rooks[wh] as usize] = Rights::new() | Dimension::WhiteH;
        info.rights_masks[info.rooks[wa] as usize] = Rights::new() | Dimension::WhiteA;
        info.rights_masks[info.rooks[bh] as usize] = Rights::new() | Dimension::BlackH;
        info.rights_masks[info.rooks[ba] as usize] = Rights::new() | Dimension::BlackA;

        info
    }

    pub fn get_updates(&self, square: Square) -> Rights {
        self.rights_masks[square as usize]
    }

    pub fn rook(&self, side: Dimension) -> Square {
        self.rooks[side as usize]
    }

    pub fn attack_mask(&self, side: Dimension) -> BitBoard {
        self.attacks_mask[side as usize]
    }

    pub fn blocker_mask(&self, side: Dimension) -> BitBoard {
        self.blocker_mask[side as usize]
    }
}
