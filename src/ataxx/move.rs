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
use std::mem::MaybeUninit;

use crate::ataxx::Square;

/// Move represents an Ataxx move which can be played on the Board.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Move(u16);

impl Move {
    // Bit-widths of fields.
    const SOURCE_WIDTH: u16 = 6;
    const TARGET_WIDTH: u16 = 6;

    // Bit-masks of fields.
    const SOURCE_MASK: u16 = (1 << Move::SOURCE_WIDTH) - 1;
    const TARGET_MASK: u16 = (1 << Move::TARGET_WIDTH) - 1;

    // Bit-offsets of fields.
    const SOURCE_OFFSET: u16 = 0;
    const TARGET_OFFSET: u16 = Move::SOURCE_OFFSET + Move::SOURCE_WIDTH;

    /// NULL Move represents an invalid move.
    pub const NULL: Move = Move(1 << 15);
    /// PASS Move represents a no move, where only the side to move changes.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let board = Board::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    /// let old_pos = board.position();
    /// let new_pos = old_pos.after_move(Move::PASS);
    ///
    /// assert_eq!(old_pos.bitboard(Color::Black), new_pos.bitboard(Color::Black));
    /// assert_eq!(old_pos.bitboard(Color::White), new_pos.bitboard(Color::White));
    /// assert_eq!(old_pos.side_to_move, !new_pos.side_to_move);
    /// ```
    pub const PASS: Move = Move(1 << 15 | 1 << 14);

    /// new_single returns a new singular Move, where a piece is cloned to its
    /// target Square. For a singular Move, [`Move::source`] and [`Move::target`]
    /// are equal since the source Square is irrelevant to the Move.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mov = Move::new_single(Square::A1);
    ///
    /// assert_eq!(mov.source(), mov.target());
    /// assert_eq!(mov.target(), Square::A1);
    /// ```
    #[inline(always)]
    pub fn new_single(square: Square) -> Move {
        Move::new(square, square)
    }

    /// new returns a new jump Move from the given source Square to the given
    /// target Square. These Squares can be recovered with the [`Move::source`] and
    /// [`Move::target`] methods respectively.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mov = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(mov.source(), Square::A1);
    /// assert_eq!(mov.target(), Square::A3);
    /// ```
    #[inline(always)]
    #[rustfmt::skip]
    pub fn new(source: Square, target: Square) -> Move {
		Move(
			(source as u16) << Move::SOURCE_OFFSET |
			(target as u16) << Move::TARGET_OFFSET
		)
    }

    /// Source returns the source Square of the moving piece. This is equal to the
    /// target Square if the given Move is of singular type.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mov = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(mov.source(), Square::A1);
    /// ```
    #[inline(always)]
    #[rustfmt::skip]
    pub fn source(self) -> Square {
        Square::try_from(
            (self.0 >> Move::SOURCE_OFFSET) & Move::SOURCE_MASK
        ).unwrap()
    }

    /// Target returns the target Square of the moving piece.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mov = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(mov.target(), Square::A3);
    /// ```
    #[inline(always)]
    #[rustfmt::skip]
    pub fn target(self) -> Square {
        Square::try_from(
            (self.0 >> Move::TARGET_OFFSET) & Move::TARGET_MASK
        ).unwrap()
    }

    /// is_single checks if the given Move is singular in nature. The result of this
    /// function for [`Move::NULL`] and [`Move::PASS`] is undefined.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let sing = Move::new_single(Square::A1);
    /// let jump = Move::new(Square::A1, Square::A3);
    ///
    /// assert!(sing.is_single());
    /// assert!(!jump.is_single());
    /// ```
    #[inline(always)]
    pub fn is_single(self) -> bool {
        self.source() == self.target()
    }
}

impl fmt::Display for Move {
    /// Display formats the given Move in a human-readable manner. The format used
    /// for displaying jump moves is `<source><target>`, while a singular Move is
    /// formatted as `<target>`. For the formatting of `<source>` and `<target>`,
    /// refer to [`Square::fmt`]. [`Move::NULL`] is  formatted as `null`, while
    /// [`Move::PASS`] is formatted as `0000`.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let null = Move::NULL;
    /// let pass = Move::PASS;
    /// let sing = Move::new_single(Square::A1);
    /// let jump = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(null.to_string(), "null");
    /// assert_eq!(pass.to_string(), "0000");
    /// assert_eq!(sing.to_string(), "a1");
    /// assert_eq!(jump.to_string(), "a1a3");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Move::NULL {
            write!(f, "null")
        } else if *self == Move::PASS {
            write!(f, "0000")
        } else if self.is_single() {
            write!(f, "{}", self.source())
        } else {
            write!(f, "{}{}", self.source(), self.target())
        }
    }
}

impl fmt::Debug for Move {
    /// Debug formats the given Move into a human-readable debug string. It uses
    /// [`Move::fmt`] under the hood for formatting the Move.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// MoveStore is a trait implemented by types which are able to store [Move]s inside
/// themselves and are thus usable in move-generation methods in [Board] like
/// [`Board::generate_moves_into<T>`](ataxx::Board::generate_moves_into<T>).
pub trait MoveStore {
    /// push adds the given Move to the MoveStore.
    fn push(&mut self, m: Move);

    /// len returns the number of [Move]s stored in the MoveStore.
    fn len(&self) -> usize;

    /// is_empty checks if no [Move]s are stored in the MoveStore.
    fn is_empty(&self) -> bool;
}

/// MoveList is a basic implementation of [`MoveStore`] that is used to allow users
/// to utilize move-generation methods without having to implement a [MoveStore] by
/// themselves. It also has utility methods other than the [`MoveStore`] trait.
pub struct MoveList {
    // A possibly uninitialized array of Moves. A fixed size array is used to allow
    // storage in the stack and thus provides more speed than a dynamic array.
    list: [MaybeUninit<Move>; 256],
    length: usize,
}

impl MoveList {
    /// new creates an empty MoveList ready for use.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let movelist = MoveList::new();
    ///
    /// assert!(movelist.is_empty());
    /// ```
    #[allow(clippy::new_without_default)]
    pub fn new() -> MoveList {
        MoveList {
            // Initialize an un-initialized array :3
            list: [MaybeUninit::uninit(); 256],
            length: 0,
        }
    }

    /// at returns the Move stored at the given index. This operation is defined and
    /// valid if and only if the 0 <= index <= MoveList length.
    pub fn at(&self, n: usize) -> Move {
        debug_assert!(n < self.len());

        // It is same to assume that the memory is initialized since the length of
        // the MoveList can only be increased by pushing Moves into it.
        unsafe { self.list[n].assume_init() }
    }

    /// iter returns an [Iterator] which iterates over the moves in the MoveList.
    pub fn iter(&self) -> MoveListIterator {
        MoveListIterator {
            list: self,
            current: 0,
        }
    }
}

// Implement the MoveStore trait to allow usage in move-generation functions.
impl MoveStore for MoveList {
    /// push adds the given move to the MoveList.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mov = Move::new(Square::A1, Square::A3);
    ///
    /// let mut movelist = MoveList::new();
    /// movelist.push(mov);
    ///
    /// assert_eq!(movelist.at(0), mov);
    /// assert_eq!(movelist.len(), 1);
    /// ```
    fn push(&mut self, m: Move) {
        self.list[self.length] = MaybeUninit::new(m);
        self.length += 1;
    }

    /// len returns the length of the MoveList.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mut movelist = MoveList::new();
    ///
    /// movelist.push(Move::new_single(Square::A1));
    /// movelist.push(Move::new_single(Square::A2));
    /// movelist.push(Move::new_single(Square::A3));
    ///
    /// assert_eq!(movelist.len(), 3);
    /// ```
    fn len(&self) -> usize {
        self.length
    }

    /// is_empty checks if the MoveList is empty, i.e its length is 0.
    ///
    fn is_empty(&self) -> bool {
        self.length == 0
    }
}

/// MoveListIterator implements an [Iterator] for a [MoveList].
pub struct MoveListIterator<'a> {
    list: &'a MoveList,
    current: usize,
}

impl Iterator for MoveListIterator<'_> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;
        if self.current <= self.list.len() {
            Some(self.list.at(self.current - 1))
        } else {
            None
        }
    }
}
