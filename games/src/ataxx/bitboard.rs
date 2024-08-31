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
use std::ops::Sub;

use derive_more::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, ShlAssign, Shr, ShrAssign,
    SubAssign,
};
use num_derive::FromPrimitive;
use strum::IntoEnumIterator;

use crate::ataxx::{File, Rank, Square};
use crate::interface::{BitBoardType, SquareType};

/// BitBoard represents a set of squares as a 64 bit bitset.
/// A BitBoard contains a Square if the 1 << square as usize
/// bit is set in the BitBoard.
///
/// Methods and overloaded operators provide support for
/// various set operations on the BitBoard.
/// ```
/// use ataxx::*;
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
#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    FromPrimitive,
    BitOr,
    BitAnd,
    BitXor,
    Shl,
    Shr,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
    SubAssign,
)]
pub struct BitBoard(pub u64);

/// bitboard is a macro which allows creation of BitBoard values from their
/// graphical representation with (.)s and (X)s inside the macro call.
/// ```
/// use ataxx::*;
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
#[allow(unused_imports)]
pub use bitboard;

impl BitBoardType for BitBoard {
    type Square = Square;

    const EMPTY: Self = BitBoard(0);
    const UNIVERSE: Self = BitBoard(0x1ffffffffffff);
    const FIRST_FILE: Self = BitBoard(0x0040810204081);
    const FIRST_RANK: Self = BitBoard(0x000000000007f);

    fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    fn trailing_zeros(&self) -> u32 {
        self.0.trailing_zeros()
    }

    fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros()
    }
}

// Iterator trait allows BitBoard to be used in a for loop.
impl Iterator for BitBoard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_lsb()
    }
}

impl Sub<usize> for BitBoard {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs as u64)
    }
}

impl From<u64> for BitBoard {
    fn from(num: u64) -> Self {
        BitBoard(num)
    }
}

impl From<BitBoard> for u64 {
    fn from(value: BitBoard) -> Self {
        value.0
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
