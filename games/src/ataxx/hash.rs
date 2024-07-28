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

use std::{fmt, ops};

use crate::ataxx::{BitBoard, Color};

/// Hash represents the semi-unique checksum of a Position used to efficiently
/// check for Position equality. Some properties of a Hash include determinism,
/// uniform distribution, avalanche effect, and collision resistance.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Hash(u64);

impl Hash {
    /// ZERO represents a usable zero or null Hash value.
    pub const ZERO: Hash = Hash(0);

    /// new creates a new Hash from the given black and white piece BitBoards.
    /// This function is used in the backend by Position, and it is usually
    /// unnecessary for it to be used explicitly by end-users. new doesn't take
    /// the blocker configuration into account since that remains unchanged
    /// throughout an ataxx game.
    pub fn new(black: BitBoard, white: BitBoard, stm: Color) -> Hash {
        let a = black.into();
        let b = white.into();

        // Currently, an 2^-63-almost delta universal hash function, based on
        // https://eprint.iacr.org/2011/116.pdf by Long Hoang Nguyen and Andrew
        // William Roscoe is used to create the Hash. This may change in the future.

        // 3 64-bit integer constants used in the hash function.
        const X: u64 = 6364136223846793005;
        const Y: u64 = 1442695040888963407;
        const Z: u64 = 2305843009213693951;

        // xa + yb + floor(ya/2^64) + floor(zb/2^64)
        // floor(pq/2^64) is essentially getting the top 64 bits of p*q.
        let part_1 = X.wrapping_mul(a); // xa
        let part_2 = Y.wrapping_mul(b); // yb
        let part_3 = (Y as u128 * a as u128) >> 64; // floor(ya/2^64) = ya >> 64
        let part_4 = (Z as u128 * b as u128) >> 64; // floor(zb/2^64) = zb >> 64

        // add the parts together and return the resultant hash.
        let hash = part_1
            .wrapping_add(part_2)
            .wrapping_add(part_3 as u64)
            .wrapping_add(part_4 as u64);

        // The Hash is bitwise complemented if the given side to move is Black.
        // Therefore, if two Positions only differ in side to move,
        // `a.Hash == !b.Hash`.
        if stm == Color::Black {
            Hash(!hash)
        } else {
            Hash(hash)
        }
    }
}

impl From<Hash> for u64 {
    fn from(value: Hash) -> Self {
        value.0
    }
}

impl ops::Not for Hash {
    type Output = Self;

    /// Not operator (!) switches the side to move for the Hash.
    fn not(self) -> Self::Output {
        Hash(!self.0)
    }
}

impl fmt::Display for Hash {
    /// Display allows Hash to be formatted in a human-readable form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

impl fmt::Debug for Hash {
    /// Debug allows Hash to be formatted in a human-readable form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
