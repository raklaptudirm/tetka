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

use std::fmt;
use std::mem::MaybeUninit;

use crate::ataxx::Square;

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
    pub const PASS: Move = Move(1 << 15 | 1 << 14);

    #[inline(always)]
    pub fn new_single(square: Square) -> Move {
        Move::new(square, square)
    }

    #[inline(always)]
    #[rustfmt::skip]
    pub fn new(source: Square, target: Square) -> Move {
		Move(
			(source as u16) << Move::SOURCE_OFFSET |
			(target as u16) << Move::TARGET_OFFSET
		)
    }

    #[inline(always)]
    #[rustfmt::skip]
    pub fn source(self) -> Square {
        Square::try_from(
            (self.0 >> Move::SOURCE_OFFSET) & Move::SOURCE_MASK
        ).unwrap()
    }

    #[inline(always)]
    #[rustfmt::skip]
    pub fn target(self) -> Square {
        Square::try_from(
            (self.0 >> Move::TARGET_OFFSET) & Move::TARGET_MASK
        ).unwrap()
    }

    #[inline(always)]
    pub fn is_single(self) -> bool {
        self.source() == self.target()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Move::NULL {
            write!(f, "0000")
        } else if self.is_single() {
            write!(f, "{}", self.source())
        } else {
            write!(f, "{}{}", self.source(), self.target())
        }
    }
}

pub trait MoveStore {
    fn push(&mut self, m: Move);
    fn len(&self) -> usize;
}

pub struct MoveList {
    list: [MaybeUninit<Move>; 256],
    length: usize,
}

impl MoveList {
    #[allow(clippy::new_without_default)]
    pub fn new() -> MoveList {
        MoveList {
            list: [MaybeUninit::uninit(); 256],
            length: 0,
        }
    }

    pub const fn at(&self, n: usize) -> Move {
        unsafe { self.list[n].assume_init() }
    }

    pub fn iter(&self) -> MoveListIterator {
        MoveListIterator {
            list: self,
            current: 0,
        }
    }
}

impl MoveStore for MoveList {
    fn push(&mut self, m: Move) {
        self.list[self.length] = MaybeUninit::new(m);
        self.length += 1;
    }

    fn len(&self) -> usize {
        self.length
    }
}

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
