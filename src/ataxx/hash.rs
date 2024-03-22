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

use std::{fmt::Display, ops};

use crate::{
    ataxx::Color,
    util::type_macros,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Hash(pub u64);

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

impl Hash {
    // new implements an 2^-63-almost delta universal hash function, based on
    // https://eprint.iacr.org/2011/116.pdf by Long Hoang Nguyen et al.
    pub fn new(a: u64, b: u64) -> Hash {
        // 3 64-bit integer constants used in the hash function.
        const X: u64 = 6364136223846793005;
        const Y: u64 = 1442695040888963407;
        const Z: u64 = 2305843009213693951;


        // xa + yb + floor(ya/2^64) + floor(zb/2^64)
        // floor(pq/2^64) is essentially getting the top 64 bits of p*q.
        let part_1 = X.wrapping_mul(a); // xa
        let part_2 = Y.wrapping_mul(b); // yb
        let part_3 = ((Y as u128 * a as u128) >> 64) as u64; // floor(ya/2^64)
        let part_4 = ((Z as u128 * b as u128) >> 64) as u64; // floor(zb/2^64)

        // add the parts together and return the resultant hash.
        Hash(part_1.wrapping_add(part_2).wrapping_add(part_3).wrapping_add(part_4))
    }

    // perspective returns the Hash from the perspective of the given Color.
    // The Hash is bitwise complemented if the given Color is Black.
    pub fn perspective(&self, color: Color) -> Hash {
        match color {
            Color::Black => !*self,
            _ => *self,
        }
    }
}

type_macros::impl_unary_ops_for_tuple! {
    for Hash:

    ops::Not, not, !;
}
