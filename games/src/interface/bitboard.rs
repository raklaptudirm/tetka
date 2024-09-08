use num_traits::int::PrimInt;
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub};

use super::{RepresentableType, SquareType};

/// BitBoardType is a generalized interface implemented by BitBoards of
/// arbitrary size. This allows programs to handle BitBoards of any size with
/// generic functions using this common interface.
pub trait BitBoardType:
    Sized
    + Copy
    + Eq
    + Into<Self::Base>
    + From<Self::Square>
    + Not<Output = Self>
    + Sub<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Shl<usize, Output = Self>
    + BitOr<Self, Output = Self>
    + BitAnd<Self, Output = Self>
    + BitXor<Self, Output = Self>
    + Iterator<Item = Self::Square>
where
    Self::Base: PrimInt,
    Self::Square: SquareType,
{
    type Base;
    type Square;

    /// EMPTY is an empty Self containing no Squares.
    const EMPTY: Self;

    /// UNIVERSE is a filled Self containing all Squares.
    const UNIVERSE: Self;

    const FIRST_FILE: Self;
    const FIRST_RANK: Self;

    /// Makes a new, empty `BitBoard`.
    fn new() -> Self {
        Self::EMPTY
    }

    /// Returns `true` if `self` has no elements in `common` with other. This is
    /// equivalent to checking for an empty intersection.
    fn is_disjoint(self, other: Self) -> bool {
        (self & other).is_empty()
    }

    /// Returns true if the BitBoard is a subset of another, i.e., `other`
    /// contains at least all the values in `self`.
    fn is_subset(self, other: Self) -> bool {
        (other & !self).is_empty()
    }

    /// Returns true if the BitBoard is a superset of another, i.e., `self`
    /// contains at least all the values in `other`.
    fn is_superset(self, other: Self) -> bool {
        other.is_subset(self)
    }

    /// Returns `true` if the BitBoard contains no elements.
    fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    /// Returns the number of elements in the BitBoard.
    fn len(self) -> usize {
        self.into().count_ones() as usize
    }

    /// Returns `true` if the BitBoard contains a value.
    fn contains(self, square: Self::Square) -> bool {
        !(self & Self::from(square)).is_empty()
    }

    /// north returns a new Self with all the squares shifted to the north.
    fn north(self) -> Self {
        (self << <Self::Square as SquareType>::File::N) & Self::UNIVERSE
    }

    /// south returns a new Self with all the squares shifted to the south.
    fn south(self) -> Self {
        self >> <Self::Square as SquareType>::File::N
    }

    /// east returns a new Self with all the squares shifted to the east.
    fn east(self) -> Self {
        (self << 1)
            & (Self::UNIVERSE
                ^ unsafe { Self::file(<Self::Square as SquareType>::File::unsafe_from(0u8)) })
    }

    /// west returns a new Self with all the squares shifted to the west.
    fn west(self) -> Self {
        (self >> 1)
            & (Self::UNIVERSE
                ^ unsafe {
                    Self::file(<Self::Square as SquareType>::File::unsafe_from(
                        <Self::Square as SquareType>::File::N as u8 - 1,
                    ))
                })
    }

    /// Adds `square` to the BitBoard.
    fn insert(&mut self, square: Self::Square) {
        *self = *self | Self::from(square)
    }

    /// Removes `square` from the BitBoard.
    fn remove(&mut self, square: Self::Square) {
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
    fn retain<F: FnMut(Self::Square) -> bool>(&mut self, mut f: F) {
        for sq in *self {
            if !f(sq) {
                self.remove(sq)
            }
        }
    }

    /// Returns a BitBoard containing all the squares from the given `File`.
    fn file(file: <Self::Square as SquareType>::File) -> Self {
        Self::FIRST_FILE << file.into() as usize
    }

    /// Returns a BitBoard containing all the squares from the given `Rank`.
    fn rank(rank: <Self::Square as SquareType>::Rank) -> Self {
        Self::FIRST_RANK << (<Self::Square as SquareType>::File::N * rank.into() as usize)
    }
}
