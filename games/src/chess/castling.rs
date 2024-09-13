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

use crate::interface::{BitBoardType, RepresentableType, SquareType};

use super::{BitBoard, Color, File, Rank, Square};
use std::ops;

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Rights(pub u8);

impl Rights {
    pub const N: usize = 16;

    pub const WH: Rights =
        Rights(SideColor(Color::White, Side::H).bit_offset() as u8);
    pub const WA: Rights =
        Rights(SideColor(Color::White, Side::A).bit_offset() as u8);
    pub const BH: Rights =
        Rights(SideColor(Color::Black, Side::H).bit_offset() as u8);
    pub const BA: Rights =
        Rights(SideColor(Color::Black, Side::A).bit_offset() as u8);

    pub fn has(self, side: SideColor) -> bool {
        self.0 >> side.bit_offset() & 1 != 0
    }
}

impl From<Color> for Rights {
    fn from(color: Color) -> Self {
        Rights((1 << (4 + 1)) << color as u16)
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::Add for Rights {
    type Output = Rights;

    fn add(self, rhs: Self) -> Self::Output {
        Rights(self.0 | rhs.0)
    }
}

impl ops::Sub for Rights {
    type Output = Rights;

    fn sub(self, rhs: Self) -> Self::Output {
        Rights(self.0 & !rhs.0)
    }
}

impl ops::SubAssign for Rights {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.0
    }
}

impl ops::Add<SideColor> for Rights {
    type Output = Rights;

    fn add(self, rhs: SideColor) -> Self::Output {
        Rights(self.0 | 1 << rhs.bit_offset())
    }
}

impl ops::Sub<SideColor> for Rights {
    type Output = Rights;

    fn sub(self, rhs: SideColor) -> Self::Output {
        Rights(self.0 & !(1 << rhs.bit_offset()))
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::Add<Color> for Rights {
    type Output = Rights;

    fn add(self, rhs: Color) -> Self::Output {
        Rights(self.0 | Rights::from(rhs).0)
    }
}

impl ops::Sub<Color> for Rights {
    type Output = Rights;

    fn sub(self, rhs: Color) -> Self::Output {
        Rights(self.0 & !Rights::from(rhs).0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SideColor(pub Color, pub Side);

impl SideColor {
    pub const N: usize = 4;

    pub fn from_sqs(king_sq: Square, rook_sq: Square) -> SideColor {
        let color: Color = if king_sq.rank() == Rank::First {
            Color::White
        } else {
            Color::Black
        };

        SideColor(color, Side::from_sqs(king_sq, rook_sq))
    }

    pub fn get_targets(self) -> (Square, Square) {
        match self {
            SideColor(Color::White, Side::H) => (Square::G1, Square::F1),
            SideColor(Color::White, Side::A) => (Square::C1, Square::D1),
            SideColor(Color::Black, Side::H) => (Square::G8, Square::F8),
            SideColor(Color::Black, Side::A) => (Square::C8, Square::D8),
        }
    }

    const fn bit_offset(self) -> usize {
        let SideColor(color, side) = self;
        color as usize * Color::N + side as usize
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
    rooks: [Square; SideColor::N],
    paths: [BitBoard; SideColor::N],
    rights_masks: [Rights; Square::N],
}

impl Info {
    //pub fn from_pos_and_str() -> Info {}

    #[rustfmt::skip]
    pub fn from_squares(
        w_king: Square, w_rook_h: File, w_rook_a: File,
        b_king: Square, b_rook_h: File, b_rook_a: File,
    ) -> Info {
        let mut info = Info {
            rights: Rights(0),
            rooks: [Square::A1; SideColor::N],
            paths: [BitBoard::EMPTY; SideColor::N],
            rights_masks: [Rights::default(); Square::N],
        };

        // Get the bit offsets/indexes of each side-color.
        let wh = SideColor(Color::White, Side::H).bit_offset();
        let wa = SideColor(Color::White, Side::A).bit_offset();
        let bh = SideColor(Color::Black, Side::H).bit_offset();
        let ba = SideColor(Color::Black, Side::A).bit_offset();

        // Initialize the rook square table.
        info.rooks[wh] = Square::new(w_rook_h, Rank::First);
        info.rooks[wa] = Square::new(w_rook_a, Rank::First);
        info.rooks[bh] = Square::new(b_rook_h, Rank::Eighth);
        info.rooks[ba] = Square::new(b_rook_a, Rank::Eighth);

        // Initialize the castling path table.
        info.paths[wh] = BitBoard::between(w_king, info.rooks[wh]) | BitBoard::from(w_king);
        info.paths[wa] = BitBoard::between(w_king, info.rooks[wa]) | BitBoard::from(w_king);
        info.paths[bh] = BitBoard::between(b_king, info.rooks[bh]) | BitBoard::from(b_king);
        info.paths[ba] = BitBoard::between(b_king, info.rooks[ba]) | BitBoard::from(b_king);

        // Initialize the rights update for the king's squares.
        info.rights_masks[w_king as usize] = Rights::WH + Rights::WA;
        info.rights_masks[b_king as usize] = Rights::BH + Rights::BA;

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

    pub fn rook(&self, side: SideColor) -> Square {
        self.rooks[side.bit_offset()]
    }

    pub fn path(&self, side: SideColor) -> BitBoard {
        self.paths[side.bit_offset()]
    }
}
