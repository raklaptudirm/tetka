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
use crate::interface::{
    representable_type, set_type, RepresentableType, SetType, SquareType,
};

use super::{BitBoard, Color, File, Rank, Square};

set_type!(Rights<Dimension>: u8);

impl Rights {
    pub const WH: Rights = Rights(Dimension::WhiteH as u8);
    pub const WA: Rights = Rights(Dimension::WhiteA as u8);
    pub const BH: Rights = Rights(Dimension::BlackH as u8);
    pub const BA: Rights = Rights(Dimension::BlackA as u8);
}

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

impl Info {
    #[rustfmt::skip]
    pub fn from_squares(
        w_king: Square, w_rook_h: File, w_rook_a: File,
        b_king: Square, b_rook_h: File, b_rook_a: File,
    ) -> Info {
        let mut info = Info {
            rights: Rights::WH,
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
        info.attacks_mask[wh] = blocker_mask(w_king, info.rooks[wh], ends::WHITE_KING_H, ends::WHITE_ROOK_H);
        info.attacks_mask[wa] = blocker_mask(w_king, info.rooks[wa], ends::WHITE_KING_A, ends::WHITE_ROOK_A);
        info.attacks_mask[bh] = blocker_mask(b_king, info.rooks[bh], ends::BLACK_KING_H, ends::BLACK_ROOK_H);
        info.attacks_mask[ba] = blocker_mask(b_king, info.rooks[ba], ends::BLACK_KING_A, ends::BLACK_ROOK_A);

        info.blocker_mask[wh] = BitBoard::between2(w_king, ends::WHITE_KING_H);
        info.blocker_mask[wa] = BitBoard::between2(w_king, ends::WHITE_KING_A);
        info.blocker_mask[bh] = BitBoard::between2(b_king, ends::BLACK_KING_H);
        info.blocker_mask[ba] = BitBoard::between2(b_king, ends::BLACK_KING_A);

        fn blocker_mask(king: Square, rook: Square, king_end: Square, rook_end: Square) -> BitBoard {
            (BitBoard::between2(king, king_end) | BitBoard::between2(rook, rook_end)) - (BitBoard::from(king) | BitBoard::from(rook))
        }

        // Initialize the rights update for the king's squares.
        info.rights_masks[w_king as usize] = Rights::WH | Rights::WA;
        info.rights_masks[b_king as usize] = Rights::BH | Rights::BA;

        // Initialize the rights update for the rook's squares.
        info.rights_masks[w_rook_h as usize] = Rights::WH;
        info.rights_masks[w_rook_a as usize] = Rights::WA;
        info.rights_masks[b_rook_h as usize] = Rights::BH;
        info.rights_masks[b_rook_a as usize] = Rights::BA;

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
