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

use num_derive::FromPrimitive;
use strum::IntoEnumIterator;

use crate::util::type_macros;

use super::{File, Rank, Square};

/// BitBoard represents a set of squares as a 64 bit bitset.
/// A BitBoard contains a Square if the 1 << square as usize
/// bit is set in the BitBoard.
///
/// Methods and overloaded operators provide support for
/// various set operations on the BitBoard.
/// ```
/// use mexx::ataxx::*;
///
/// let a1 = BitBoard::from(Square::A1);
/// let b1 = BitBoard::from(Square::B1);
/// let c1 = BitBoard::from(Square::C1);
///
/// let x = a1 | b1;
/// let y = a1 | c1;
///
/// assert_eq!(x | y, a1 | b1 | c1); // Union
/// assert_eq!(x & y, a1);           // Intersection
/// assert_eq!(x ^ y, b1 | c1);      // Symmetric Difference
/// assert_eq!(x - y, b1);           // Difference
///
/// assert_eq!(!x, BitBoard::UNIVERSE - x); // Complement
/// ```
#[derive(Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub struct BitBoard(pub u64);

/// bitboard is a macro which allows creation of BitBoard values from their
/// graphical representation with (.)s and (X)s inside the macro call.
/// ```
/// use mexx::ataxx::*;
///
/// assert_eq!(BitBoard::file(File::B) | Square::G7, bitboard! {
///     . X . . . . X
///     . X . . . . .
///     . X . . . . .
///     . X . . . . .
///     . X . . . . .
///     . X . . . . .
///     . X . . . . .
/// });
/// ```
// Implementation derived from: https://github.com/analog-hors/cozy-chess
#[macro_export]
macro_rules! bitboard {
    (
        $a7:tt $b7:tt $c7:tt $d7:tt $e7:tt $f7:tt $g7:tt
        $a6:tt $b6:tt $c6:tt $d6:tt $e6:tt $f6:tt $g6:tt
        $a5:tt $b5:tt $c5:tt $d5:tt $e5:tt $f5:tt $g5:tt
        $a4:tt $b4:tt $c4:tt $d4:tt $e4:tt $f4:tt $g4:tt
        $a3:tt $b3:tt $c3:tt $d3:tt $e3:tt $f3:tt $g3:tt
        $a2:tt $b2:tt $c2:tt $d2:tt $e2:tt $f2:tt $g2:tt
        $a1:tt $b1:tt $c1:tt $d1:tt $e1:tt $f1:tt $g1:tt
    ) => {
        $crate::bitboard! { @__inner
            $a1 $b1 $c1 $d1 $e1 $f1 $g1
            $a2 $b2 $c2 $d2 $e2 $f2 $g2
            $a3 $b3 $c3 $d3 $e3 $f3 $g3
            $a4 $b4 $c4 $d4 $e4 $f4 $g4
            $a5 $b5 $c5 $d5 $e5 $f5 $g5
            $a6 $b6 $c6 $d6 $e6 $f6 $g6
            $a7 $b7 $c7 $d7 $e7 $f7 $g7
        }
    };
    (@__inner $($occupied:tt)*) => {{
        const BITBOARD: BitBoard = {
            let mut index = 0;
            let mut bitboard = BitBoard::EMPTY;
            $(
                if $crate::bitboard!(@__square $occupied) {
                    bitboard.0 |= 1 << index;
                }

                index += 1;
            )*
            let _ = index;
            bitboard
        };
        BITBOARD
    }};
    (@__square X) => { true };
    (@__square .) => { false };
    (@__square $token:tt) => {
        compile_error!(
            concat!(
                "Expected only `X` or `.` tokens, found `",
                stringify!($token),
                "`"
            )
        )
    };
    ($($token:tt)*) => {
        compile_error!("Expected 49 squares")
    };
}
pub use bitboard;

impl BitBoard {
    /// EMPTY is an empty BitBoard containing no Squares.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert_eq!(BitBoard::EMPTY, bitboard! {
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    /// });
    /// ```
    pub const EMPTY: BitBoard = BitBoard(0);

    /// UNIVERSE is a filled BitBoard containing all Squares.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert_eq!(BitBoard::UNIVERSE, bitboard! {
    ///     X X X X X X X
    ///     X X X X X X X
    ///     X X X X X X X
    ///     X X X X X X X
    ///     X X X X X X X
    ///     X X X X X X X
    ///     X X X X X X X
    /// });
    /// ```
    pub const UNIVERSE: BitBoard = BitBoard(0x1ffffffffffff);

    /// is_disjoint checks if the two BitBoards are disjoint, i.e. don't have
    /// any squares in common among themselves.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert!( BitBoard::UNIVERSE.is_disjoint(BitBoard::EMPTY));
    /// assert!(!BitBoard::UNIVERSE.is_disjoint(BitBoard::UNIVERSE));
    /// ```
    #[inline(always)]
    pub const fn is_disjoint(self, other: BitBoard) -> bool {
        self.0 & other.0 == BitBoard::EMPTY.0
    }

    /// is_subset checks if the given BitBoard is a subset of the target, i.e.
    /// all the squares in the target are also present in the given BitBoard.
    /// ```
    /// use std::sync::atomic::fence;
    /// use mexx::ataxx::*;
    ///
    /// assert!( BitBoard::UNIVERSE.is_subset(BitBoard::EMPTY));
    /// assert!(!BitBoard::EMPTY.is_subset(BitBoard::UNIVERSE));
    /// ```
    #[inline(always)]
    pub const fn is_subset(self, other: BitBoard) -> bool {
        other.0 & !self.0 == BitBoard::EMPTY.0
    }

    /// is_superset checks if the given BitBoard is a superset of the target, i.e.
    /// all the squares in the given BitBoard are also present in the target.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert!(!BitBoard::UNIVERSE.is_superset(BitBoard::EMPTY));
    /// assert!( BitBoard::EMPTY.is_superset(BitBoard::UNIVERSE));
    /// ```
    #[inline(always)]
    pub const fn is_superset(self, other: BitBoard) -> bool {
        other.is_subset(self)
    }

    /// is_empty checks if the target BitBoard is empty.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert!( BitBoard::EMPTY.is_empty());
    /// assert!(!BitBoard::UNIVERSE.is_empty());
    /// ```
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == BitBoard::EMPTY.0
    }

    /// cardinality returns the number of Squares present in the BitBoard.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert_eq!(BitBoard::EMPTY.cardinality(), 0);
    /// assert_eq!(BitBoard::UNIVERSE.cardinality(), Square::N);
    /// ```
    pub const fn cardinality(self) -> usize {
        self.0.count_ones() as usize
    }

    /// contains checks if the BitBoard contains the given Square.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert!(!BitBoard::EMPTY.contains(Square::A1));
    /// assert!( BitBoard::UNIVERSE.contains(Square::A1));
    /// ```
    #[inline(always)]
    pub const fn contains(self, square: Square) -> bool {
        self.0 & (1 << square as u64) != BitBoard::EMPTY.0
    }

    /// north returns a new BitBoard with all the squares shifted to the north.
    /// ```
    /// use mexx::ataxx::*;
    /// assert_eq!(BitBoard::rank(Rank::First).north(), bitboard! {
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     X X X X X X X
    ///     . . . . . . .
    /// });
    /// ```
    pub const fn north(self) -> BitBoard {
        BitBoard((self.0 << File::N) & BitBoard::UNIVERSE.0)
    }

    /// south returns a new BitBoard with all the squares shifted to the south.
    /// ```
    /// use mexx::ataxx::*;
    /// assert_eq!(BitBoard::rank(Rank::Seventh).south(), bitboard! {
    ///     . . . . . . .
    ///     X X X X X X X
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    /// });
    /// ```
    pub const fn south(self) -> BitBoard {
        BitBoard(self.0 >> File::N)
    }

    /// east returns a new BitBoard with all the squares shifted to the east.
    /// ```
    /// use mexx::ataxx::*;
    /// assert_eq!(BitBoard::file(File::A).east(), bitboard! {
    ///     . X . . . . .
    ///     . X . . . . .
    ///     . X . . . . .
    ///     . X . . . . .
    ///     . X . . . . .
    ///     . X . . . . .
    ///     . X . . . . .
    /// });
    /// ```
    pub const fn east(self) -> BitBoard {
        BitBoard((self.0 << 1) & (BitBoard::UNIVERSE.0 ^ BitBoard::file(File::A).0))
    }

    /// west returns a new BitBoard with all the squares shifted to the west.
    /// ```
    /// use mexx::ataxx::*;
    /// assert_eq!(BitBoard::file(File::G).west(), bitboard! {
    ///     . . . . . X .
    ///     . . . . . X .
    ///     . . . . . X .
    ///     . . . . . X .
    ///     . . . . . X .
    ///     . . . . . X .
    ///     . . . . . X .
    /// });
    /// ```
    pub const fn west(self) -> BitBoard {
        BitBoard((self.0 >> 1) & (BitBoard::UNIVERSE.0 ^ BitBoard::file(File::G).0))
    }

    /// insert puts the given Square into the BitBoard.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mut bb = BitBoard::EMPTY;
    /// bb.insert(Square::A1);
    /// assert!(bb.contains(Square::A1));
    /// ```
    #[inline(always)]
    pub fn insert(&mut self, square: Square) {
        self.0 |= BitBoard::from(square).0
    }

    /// remove removes the given Square from the BitBoard.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mut bb = BitBoard::UNIVERSE;
    /// bb.remove(Square::A1);
    /// assert!(!bb.contains(Square::A1));
    /// ```
    #[inline(always)]
    pub fn remove(&mut self, square: Square) {
        self.0 &= !BitBoard::from(square).0
    }

    /// pop_lsb pops the least significant Square from the BitBoard, i.e. it
    /// removes the lsb from the BitBoard and returns its value.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mut bb = BitBoard::UNIVERSE;
    /// assert_eq!(bb.pop_lsb(), Square::A1);
    /// assert!(!bb.contains(Square::A1));
    /// ```
    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Square {
        let lsb = self.lsb();
        self.0 &= self.0 - 1;

        lsb
    }

    /// pop_msb pops the most significant Square from the BitBoard i.e. it
    /// removes the msb from the BitBoard and returns its value.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mut bb = BitBoard::UNIVERSE;
    /// assert_eq!(bb.pop_msb(), Square::G7);
    /// assert!(!bb.contains(Square::G7));
    /// ```
    #[inline(always)]
    pub fn pop_msb(&mut self) -> Square {
        let msb = self.msb();
        self.0 ^= BitBoard::from(msb).0;

        msb
    }

    /// get_lsb returns the least significant Square from the BitBoard.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert_eq!(BitBoard::UNIVERSE.lsb(), Square::A1);
    /// ```
    #[inline(always)]
    pub fn lsb(self) -> Square {
        Square::try_from(self.0.trailing_zeros()).unwrap()
    }

    /// get_msb returns the most significant Square from the BitBoard.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert_eq!(BitBoard::UNIVERSE.msb(), Square::G7);
    /// ```
    #[inline(always)]
    pub fn msb(self) -> Square {
        Square::try_from(63 - self.0.leading_zeros()).unwrap()
    }
}

// Iterator trait allows BitBoard to be used in a for loop.
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

// From trait allows a square to be converted into it's BitBoard representation.
impl From<Square> for BitBoard {
    fn from(square: Square) -> Self {
        BitBoard(1 << square as u64)
    }
}

// Not(!)/Complement operation implementation for BitBoard.
impl ops::Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        // ! will set the unused bits so remove them with an &.
        BitBoard(!self.0) & BitBoard::UNIVERSE
    }
}

// Implementation of various binary operations.
type_macros::impl_binary_ops_for_tuple! {
    for BitBoard:

    ops::BitOr, bitor, |;
    ops::BitXor, bitxor, ^;
    ops::BitAnd, bitand, &;

    ops::Shl, shl, <<;
    ops::Shr, shr, >>;
}

// Implementation of various assignment operations.
type_macros::impl_assign_ops_for_tuple! {
    for BitBoard:

    ops::SubAssign, sub_assign, -;

    ops::BitOrAssign, bitor_assign, |;
    ops::BitXorAssign, bitxor_assign, ^;
    ops::BitAndAssign, bitand_assign, &;

    ops::ShlAssign, shl_assign, <<;
    ops::ShrAssign, shr_assign, >>;
}

// Implementation of subtraction(removal) of BitBoards.
#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::Sub for BitBoard {
    type Output = BitBoard;

    fn sub(self, rhs: Self) -> Self::Output {
        self & !rhs
    }
}

// Implementation of |(or)/set-union of a BitBoard with a Square.
#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::BitOr<Square> for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Square) -> Self::Output {
        self | BitBoard::from(rhs)
    }
}

// Implementation of -(subtraction)/set-removal of a BitBoard with a Square.
impl ops::Sub<Square> for BitBoard {
    type Output = BitBoard;

    fn sub(self, rhs: Square) -> Self::Output {
        self & !BitBoard::from(rhs)
    }
}

// Display a BitBoard as ASCII art with 0s and 1s.
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

impl fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl BitBoard {
    /// file returns a BitBoard containing all the squares from the given File.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert_eq!(BitBoard::file(File::A), bitboard! {
    ///     X . . . . . .
    ///     X . . . . . .
    ///     X . . . . . .
    ///     X . . . . . .
    ///     X . . . . . .
    ///     X . . . . . .
    ///     X . . . . . .
    /// });
    /// ```
    pub const fn file(file: File) -> BitBoard {
        BitBoard::FILE[file as usize]
    }

    /// rank returns a BitBoard containing all the squares from the given Rank.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert_eq!(BitBoard::rank(Rank::First), bitboard! {
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     X X X X X X X
    /// });
    /// ```
    pub const fn rank(rank: Rank) -> BitBoard {
        BitBoard::RANK[rank as usize]
    }

    /// single returns the targets of a singular Move from the given Square.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert_eq!(BitBoard::single(Square::A1), bitboard! {
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     X X . . . . .
    ///     . X . . . . .
    /// });
    /// ```
    pub const fn single(square: Square) -> BitBoard {
        BitBoard::SINGLES[square as usize]
    }

    /// double returns the targets of a jump Move from the given Square.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// assert_eq!(BitBoard::double(Square::A1), bitboard! {
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     X X X . . . .
    ///     . . X . . . .
    ///     . . X . . . .
    /// });
    /// ```
    pub const fn double(square: Square) -> BitBoard {
        BitBoard::DOUBLES[square as usize]
    }
}

// Private constants wrapped in methods for indexing.
impl BitBoard {
    const FILE: [BitBoard; File::N] = [
        BitBoard(0x0040810204081),
        BitBoard(0x0081020408102),
        BitBoard(0x0102040810204),
        BitBoard(0x0204081020408),
        BitBoard(0x0408102040810),
        BitBoard(0x0810204081020),
        BitBoard(0x1020408102040),
    ];

    const RANK: [BitBoard; Rank::N] = [
        BitBoard(0x000000000007f),
        BitBoard(0x0000000003f80),
        BitBoard(0x00000001fc000),
        BitBoard(0x000000fe00000),
        BitBoard(0x00007f0000000),
        BitBoard(0x003f800000000),
        BitBoard(0x1fc0000000000),
    ];

    const SINGLES: [BitBoard; Square::N] = [
        BitBoard(0x0000000000182),
        BitBoard(0x0000000000385),
        BitBoard(0x000000000070a),
        BitBoard(0x0000000000e14),
        BitBoard(0x0000000001c28),
        BitBoard(0x0000000003850),
        BitBoard(0x0000000003020),
        BitBoard(0x000000000c103),
        BitBoard(0x000000001c287),
        BitBoard(0x000000003850e),
        BitBoard(0x0000000070a1c),
        BitBoard(0x00000000e1438),
        BitBoard(0x00000001c2870),
        BitBoard(0x0000000181060),
        BitBoard(0x0000000608180),
        BitBoard(0x0000000e14380),
        BitBoard(0x0000001c28700),
        BitBoard(0x0000003850e00),
        BitBoard(0x00000070a1c00),
        BitBoard(0x000000e143800),
        BitBoard(0x000000c083000),
        BitBoard(0x000003040c000),
        BitBoard(0x0000070a1c000),
        BitBoard(0x00000e1438000),
        BitBoard(0x00001c2870000),
        BitBoard(0x00003850e0000),
        BitBoard(0x000070a1c0000),
        BitBoard(0x0000604180000),
        BitBoard(0x0001820600000),
        BitBoard(0x0003850e00000),
        BitBoard(0x00070a1c00000),
        BitBoard(0x000e143800000),
        BitBoard(0x001c287000000),
        BitBoard(0x003850e000000),
        BitBoard(0x003020c000000),
        BitBoard(0x00c1030000000),
        BitBoard(0x01c2870000000),
        BitBoard(0x03850e0000000),
        BitBoard(0x070a1c0000000),
        BitBoard(0x0e14380000000),
        BitBoard(0x1c28700000000),
        BitBoard(0x1810600000000),
        BitBoard(0x0081800000000),
        BitBoard(0x0143800000000),
        BitBoard(0x0287000000000),
        BitBoard(0x050e000000000),
        BitBoard(0x0a1c000000000),
        BitBoard(0x1438000000000),
        BitBoard(0x0830000000000),
    ];
    const DOUBLES: [BitBoard; Square::N] = [
        BitBoard(0x000000001c204),
        BitBoard(0x000000003c408),
        BitBoard(0x000000007c891),
        BitBoard(0x00000000f9122),
        BitBoard(0x00000001f2244),
        BitBoard(0x00000001e0408),
        BitBoard(0x00000001c0810),
        BitBoard(0x0000000e10204),
        BitBoard(0x0000001e20408),
        BitBoard(0x0000003e44891),
        BitBoard(0x0000007c89122),
        BitBoard(0x000000f912244),
        BitBoard(0x000000f020408),
        BitBoard(0x000000e040810),
        BitBoard(0x0000070810207),
        BitBoard(0x00000f102040f),
        BitBoard(0x00001f224489f),
        BitBoard(0x00003e448913e),
        BitBoard(0x00007c891227c),
        BitBoard(0x0000781020478),
        BitBoard(0x0000702040870),
        BitBoard(0x0003840810380),
        BitBoard(0x0007881020780),
        BitBoard(0x000f912244f80),
        BitBoard(0x001f224489f00),
        BitBoard(0x003e448913e00),
        BitBoard(0x003c081023c00),
        BitBoard(0x0038102043800),
        BitBoard(0x01c204081c000),
        BitBoard(0x03c408103c000),
        BitBoard(0x07c891227c000),
        BitBoard(0x0f912244f8000),
        BitBoard(0x1f224489f0000),
        BitBoard(0x1e040811e0000),
        BitBoard(0x1c081021c0000),
        BitBoard(0x0102040e00000),
        BitBoard(0x0204081e00000),
        BitBoard(0x0448913e00000),
        BitBoard(0x0891227c00000),
        BitBoard(0x112244f800000),
        BitBoard(0x020408f000000),
        BitBoard(0x040810e000000),
        BitBoard(0x0102070000000),
        BitBoard(0x02040f0000000),
        BitBoard(0x04489f0000000),
        BitBoard(0x08913e0000000),
        BitBoard(0x11227c0000000),
        BitBoard(0x0204780000000),
        BitBoard(0x0408700000000),
    ];
}
