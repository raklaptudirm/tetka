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

use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

use super::RepresentableType;

use num_traits::PrimInt;

/// SetTypes behave like a set of a given type of objects.
///
/// The API has been adapted from set-like types from the standard libary.
pub trait SetType<B: PrimInt, E: RepresentableType<u8>>:
    Sized
    + Copy
    + Eq
    + Into<B>
    + From<E>
    + Not<Output = Self>
    + Shr<usize, Output = Self>
    + Shl<usize, Output = Self>
    + BitOr<Self, Output = Self>
    + BitAnd<Self, Output = Self>
    + BitXor<Self, Output = Self>
    + Iterator<Item = E>
{
    /// The null/empty set containing no elements.
    const EMPTY: Self;

    /// The universal set containing all elements.
    const UNIVERSE: Self;

    /// Makes a new, empty `BitBoard`.
    fn new() -> Self {
        Self::EMPTY
    }

    /// Returns `true` if `self` has no elements in `common` with other. This is
    /// equivalent to checking for an empty intersection.
    #[must_use]
    fn is_disjoint(self, other: Self) -> bool {
        (self & other).is_empty()
    }

    /// Returns true if the BitBoard is a subset of another, i.e., `other`
    /// contains at least all the values in `self`.
    #[must_use]
    fn is_subset(self, other: Self) -> bool {
        (other & !self).is_empty()
    }

    /// Returns true if the BitBoard is a superset of another, i.e., `self`
    /// contains at least all the values in `other`.
    #[must_use]
    fn is_superset(self, other: Self) -> bool {
        other.is_subset(self)
    }

    /// Returns `true` if the BitBoard contains no elements.
    #[must_use]
    fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    /// Returns the number of elements in the BitBoard.
    #[must_use]
    fn len(self) -> usize {
        self.into().count_ones() as usize
    }

    /// Returns `true` if the BitBoard contains a value.
    #[must_use]
    fn contains(self, square: E) -> bool {
        !(self & Self::from(square)).is_empty()
    }

    /// Adds `square` to the BitBoard.
    fn insert(&mut self, square: E) {
        *self = *self | Self::from(square)
    }

    /// Removes `square` from the BitBoard.
    fn remove(&mut self, square: E) {
        *self = *self & !Self::from(square)
    }

    /// Clears the BitBoard, removing all elements.
    fn clear(&mut self) {
        *self = Self::EMPTY
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `s` for which `f(s)` returns `false`.
    /// The elements are visited in ascending order.
    fn retain<F: FnMut(E) -> bool>(&mut self, mut f: F) {
        for sq in *self {
            if !f(sq) {
                self.remove(sq)
            }
        }
    }
}
