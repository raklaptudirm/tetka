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
use std::ops;

use super::{ Square, File, Rank };

use crate::util::type_macros;

use num_derive::FromPrimitive;
use strum::IntoEnumIterator;

/// BitBoard represents a set of squares as a 64 bit bitset.
#[derive(Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub struct BitBoard(pub u64);

impl BitBoard {
    /// EMPTY represents an empty BitBoard.
    pub const EMPTY: BitBoard = BitBoard(0);

    // UNIVERSE represents a filled BitBoard.
    pub const UNIVERSE: BitBoard = BitBoard(!0);

    // is_disjoint checks if the two BitBoards don't have any squares in common.
    #[inline(always)]
    pub const fn is_disjoint(self, other: BitBoard) -> bool {
        self.0 & other.0 == BitBoard::EMPTY.0
    }

    // is_subset checks if the given BitBoard is a subset of the target.
    #[inline(always)]
    pub const fn is_subset(self, other: BitBoard) -> bool {
        other.0 & !self.0 == BitBoard::EMPTY.0
    }

    // is_superset checks if the given BitBoard is a superset of the target.
    #[inline(always)]
    pub const fn is_superset(self, other: BitBoard) -> bool {
        other.is_subset(self)
    }

    // is_empty checks if the target BitBoard is empty.
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == BitBoard::EMPTY.0
    }

    // cardinality returns the number of elements in the BitBoard.
    pub const fn cardinality(self) -> u32 {
        self.0.count_ones()
    }

    /// contains checks if the BitBoard contains the given Square.
    #[inline(always)]
    pub const fn contains(self, square: Square) -> bool {
        self.0 & (1 << square as u64) != BitBoard::EMPTY.0
    }

    // north returns a new BitBoard with all the squares shifted to the north.
    pub const fn north(self) -> BitBoard {
        BitBoard((self.0 << 8) & 0x7f7f7f7f7f7f7f)
    }

    // south returns a new BitBoard with all the squares shifted to the south.
    pub const fn south(self) -> BitBoard {
        BitBoard(self.0  >> 8)
    }

    // east returns a new BitBoard with all the squares shifted to the east.
    pub const fn east(self) -> BitBoard {
        BitBoard((self.0 << 1) & 0x7e7e7e7e7e7e7e)
    }

    // west returns a new BitBoard with all the squares shifted to the west.
    pub const fn west(self) -> BitBoard {
        BitBoard((self.0 >> 1) & 0x3f3f3f3f3f3f3f)
    }

    /// insert puts the given Square into the BitBoard.
    #[inline(always)]
    pub fn insert(&mut self, square: Square) {
        self.0 |= BitBoard::from(square).0
    }

    /// remove removes the given Square from the BitBoard.
    #[inline(always)]
    pub fn remove(&mut self, square: Square) {
        self.0 &= !BitBoard::from(square).0
    }

    /// pop_lsb pops the least significant Square from the BitBoard.
    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Square {
        let lsb = self.lsb();
        self.0 &= self.0 - 1;

        lsb
    }

    /// pop_msb pops the most significant Square from the BitBoard.
    #[inline(always)]
    pub fn pop_msb(&mut self) -> Square {
        let msb = self.msb();
        self.0 ^= BitBoard::from(msb).0;

        msb
    }

    /// get_lsb returns the least significant Square from the BitBoard.
    #[inline(always)]
    pub fn lsb(self) -> Square {
        Square::try_from(self.0.trailing_zeros()).unwrap()
    }

    /// get_msb returns the most significant Square from the BitBoard.
    #[inline(always)]
    pub fn msb(self) -> Square {
        Square::try_from(63 - self.0.leading_zeros()).unwrap()
    }
}

// various trait implementations

impl Iterator for BitBoard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.pop_lsb())
        }
    }
}

impl From<Square> for BitBoard {
    fn from(square: Square) -> Self {
        BitBoard(1 << square as u64)
    }
}

// operation implementations

// implement unary operations
type_macros::impl_unary_ops_for_tuple! {
    for BitBoard:
    ops::Not, not, !;
}

// implement binary operations
type_macros::impl_binary_ops_for_tuple! {
    for BitBoard:

    ops::BitOr, bitor, |;
    ops::BitXor, bitxor, ^;
    ops::BitAnd, bitand, &;

    ops::Shl, shl, <<;
    ops::Shr, shr, >>;
}

// implement assignment operations
type_macros::impl_assign_ops_for_tuple! {
    for BitBoard:

    ops::BitOrAssign, bitor_assign, |;
    ops::BitXorAssign, bitxor_assign, ^;
    ops::BitAndAssign, bitand_assign, &;

    ops::ShlAssign, shl_assign, <<;
    ops::ShrAssign, shr_assign, >>;
}

type_macros::impl_from_integer_for_tuple! {
    for BitBoard u64:

    // Unsigned Integers.
    usize, u8, u16, u32, u64,

    // Signed Integers.
    isize, i8, i16, i32, i64,
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::Sub for BitBoard {
    type Output = BitBoard;

    fn sub(self, rhs: Self) -> Self::Output {
        self & !rhs
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::BitOr<Square> for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Square) -> Self::Output {
        self | BitBoard::from(rhs)
    }
}

impl ops::Sub<Square> for BitBoard {
    type Output = BitBoard;

    fn sub(self, rhs: Square) -> Self::Output {
        self & !BitBoard::from(rhs)
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string_rep = String::from("");
        for rank in Rank::iter().rev() {
            for file in File::iter() {
                let square = Square::new(file, rank);
                string_rep += if self.contains(square) { "1 " } else { "0 " };
            }

            string_rep += "\n";
        }

        write!(f, "{string_rep}")
    }
}

impl BitBoard {
    pub const SINGLES: [BitBoard; Square::N] = [
        BitBoard(0x00000000000302), BitBoard(0x00000000000705), BitBoard(0x00000000000e0a), BitBoard(0x00000000001c14),
        BitBoard(0x00000000003828), BitBoard(0x00000000007050), BitBoard(0x00000000006020), BitBoard(0x00000000000000),
        BitBoard(0x00000000030203), BitBoard(0x00000000070507), BitBoard(0x000000000e0a0e), BitBoard(0x000000001c141c),
        BitBoard(0x00000000382838), BitBoard(0x00000000705070), BitBoard(0x00000000602060), BitBoard(0x00000000000000),
        BitBoard(0x00000003020300), BitBoard(0x00000007050700), BitBoard(0x0000000e0a0e00), BitBoard(0x0000001c141c00),
        BitBoard(0x00000038283800), BitBoard(0x00000070507000), BitBoard(0x00000060206000), BitBoard(0x00000000000000),
        BitBoard(0x00000302030000), BitBoard(0x00000705070000), BitBoard(0x00000e0a0e0000), BitBoard(0x00001c141c0000),
        BitBoard(0x00003828380000), BitBoard(0x00007050700000), BitBoard(0x00006020600000), BitBoard(0x00000000000000),
        BitBoard(0x00030203000000), BitBoard(0x00070507000000), BitBoard(0x000e0a0e000000), BitBoard(0x001c141c000000),
        BitBoard(0x00382838000000), BitBoard(0x00705070000000), BitBoard(0x00602060000000), BitBoard(0x00000000000000),
        BitBoard(0x03020300000000), BitBoard(0x07050700000000), BitBoard(0x0e0a0e00000000), BitBoard(0x1c141c00000000),
        BitBoard(0x38283800000000), BitBoard(0x70507000000000), BitBoard(0x60206000000000), BitBoard(0x00000000000000),
        BitBoard(0x02030000000000), BitBoard(0x05070000000000), BitBoard(0x0a0e0000000000), BitBoard(0x141c0000000000),
        BitBoard(0x28380000000000), BitBoard(0x50700000000000), BitBoard(0x20600000000000), BitBoard(0x00000000000000),
        BitBoard(0x00000000000000), BitBoard(0x00000000000000), BitBoard(0x00000000000000), BitBoard(0x00000000000000),
        BitBoard(0x00000000000000), BitBoard(0x00000000000000), BitBoard(0x00000000000000), BitBoard(0x00000000000000),
    ];
    pub const DOUBLES: [BitBoard; Square::N] = [
        BitBoard(0x00000000070404), BitBoard(0x000000000f0808), BitBoard(0x000000001f1111), BitBoard(0x000000003e2222),
        BitBoard(0x000000007c4444), BitBoard(0x00000000780808), BitBoard(0x00000000701010), BitBoard(0x00000000000000),
        BitBoard(0x00000007040404), BitBoard(0x0000000f080808), BitBoard(0x0000001f111111), BitBoard(0x0000003e222222),
        BitBoard(0x0000007c444444), BitBoard(0x00000078080808), BitBoard(0x00000070101010), BitBoard(0x00000000000000),
        BitBoard(0x00000704040407), BitBoard(0x00000f0808080f), BitBoard(0x00001f1111111f), BitBoard(0x00003e2222223e),
        BitBoard(0x00007c4444447c), BitBoard(0x00007808080878), BitBoard(0x00007010101070), BitBoard(0x00000000000000),
        BitBoard(0x00070404040700), BitBoard(0x000f0808080f00), BitBoard(0x001f1111111f00), BitBoard(0x003e2222223e00),
        BitBoard(0x007c4444447c00), BitBoard(0x00780808087800), BitBoard(0x00701010107000), BitBoard(0x00000000000000),
        BitBoard(0x07040404070000), BitBoard(0x0f0808080f0000), BitBoard(0x1f1111111f0000), BitBoard(0x3e2222223e0000),
        BitBoard(0x7c4444447c0000), BitBoard(0x78080808780000), BitBoard(0x70101010700000), BitBoard(0x00000000000000),
        BitBoard(0x04040407000000), BitBoard(0x0808080f000000), BitBoard(0x1111111f000000), BitBoard(0x2222223e000000),
        BitBoard(0x4444447c000000), BitBoard(0x08080878000000), BitBoard(0x10101070000000), BitBoard(0x00000000000000),
        BitBoard(0x04040700000000), BitBoard(0x08080f00000000), BitBoard(0x11111f00000000), BitBoard(0x22223e00000000),
        BitBoard(0x44447c00000000), BitBoard(0x08087800000000), BitBoard(0x10107000000000), BitBoard(0x00000000000000),
        BitBoard(0x00000000000000), BitBoard(0x00000000000000), BitBoard(0x00000000000000), BitBoard(0x00000000000000),
        BitBoard(0x00000000000000), BitBoard(0x00000000000000), BitBoard(0x00000000000000), BitBoard(0x00000000000000),
    ];
}

impl BitBoard {
    pub const fn file(file: File) -> BitBoard {
        BitBoard(BitBoard::FILE[file as usize])
    }

    pub const fn rank(rank: Rank) -> BitBoard {
        BitBoard(BitBoard::RANK[rank as usize])
    }

    const FILE: [u64; 7] = [
        0x0101010101010101,
        0x0202020202020202,
        0x0404040404040404,
        0x0808080808080808,
        0x1010101010101010,
        0x2020202020202020,
        0x4040404040404040,
    ];

    const RANK: [u64; 7] = [
        0x000000000000007f,
        0x0000000000007f00,
        0x00000000007f0000,
        0x000000007f000000,
        0x0000007f00000000,
        0x00007f0000000000,
        0x007f000000000000,
    ];
}
