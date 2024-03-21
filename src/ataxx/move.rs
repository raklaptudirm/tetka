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

use std::{fmt};
use std::mem::MaybeUninit;

use crate::ataxx;
use crate::util::type_macros;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

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

    pub const NULL: Move = Move(1 << 15);

    #[inline(always)]
    pub fn new_single(square: ataxx::Square) -> Move {
        Move::new(square, square)
    }

    pub fn new(source: ataxx::Square, target: ataxx::Square) -> Move {
        Move((source as u16) << Move::SOURCE_OFFSET | (target as u16) << Move::TARGET_OFFSET)
    }

    #[inline(always)]
    pub fn source(self) -> ataxx::Square {
        ataxx::Square::try_from((self.0 >> Move::SOURCE_OFFSET) & Move::SOURCE_MASK).unwrap()
    }

    #[inline(always)]
    pub fn target(self) -> ataxx::Square {
        ataxx::Square::try_from((self.0 >> Move::TARGET_OFFSET) & Move::TARGET_MASK).unwrap()
    }

    #[inline(always)]
    pub fn is_single(self) -> bool {
        self.source() == self.target()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default, FromPrimitive)]
#[rustfmt::skip]
pub enum MoveFlag {
    #[default] Normal, Castle, Promotion, EnPassant
}

type_macros::impl_from_integer_for_enum! {
    for MoveFlag:

    // Unsigned Integers.
    usize, MoveFlag::from_usize;
    u8, MoveFlag::from_u8; u16, MoveFlag::from_u16;
    u32, MoveFlag::from_u32; u64, MoveFlag::from_u64;

    // Signed Integers.
    isize, MoveFlag::from_isize;
    i8, MoveFlag::from_i8; i16, MoveFlag::from_i16;
    i32, MoveFlag::from_i32; i64, MoveFlag::from_i64;
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

pub struct MoveList {
    list: [MaybeUninit<Move>; 256],
    length: usize,
}

impl MoveList {
    pub fn new() -> MoveList {
            MoveList {
                list: [MaybeUninit::uninit(); 256],
                length: 0,
            }
    }
    pub const fn len(&self) -> usize {
        self.length
    }

    pub const fn at(&self, n: usize) -> Move {
        unsafe {
            self.list[n].assume_init()
        }
    }

    pub fn push(&mut self, m: Move) {
        self.list[self.length] = MaybeUninit::new(m);
        self.length += 1;
    }

    pub fn iter(&self) -> MoveListIterator {
        MoveListIterator {
            list: &self,
            current: 0,
        }
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
