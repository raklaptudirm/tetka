use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub};

use super::{RepresentableType, SquareType};

pub trait BitBoardType:
    Sized
    + Copy
    + From<Self::Square>
    + BitAnd<Self, Output = Self>
    + BitOr<Self, Output = Self>
    + Eq
    + Not<Output = Self>
    + Shr<usize, Output = Self>
    + Shl<usize, Output = Self>
    + BitXor<Self, Output = Self>
    + Sub<usize, Output = Self>
where
    <Self as BitOr<Self>>::Output: Into<Self>,
    Self::Square: SquareType,
{
    type Square;

    /// EMPTY is an empty Self containing no Squares.
    const EMPTY: Self;

    /// UNIVERSE is a filled Self containing all Squares.
    const UNIVERSE: Self;

    const FIRST_FILE: Self;
    const FIRST_RANK: Self;

    /// is_disjoint checks if the two Selfs are disjoint, i.e. don't have
    /// any squares in common among themselves.
    fn is_disjoint(self, other: Self) -> bool {
        self & other == Self::EMPTY
    }

    /// is_subset checks if the given Self is a subset of the target, i.e.
    /// all the squares in the target are also present in the given Self.
    fn is_subset(self, other: Self) -> bool {
        other & !self == Self::EMPTY
    }

    /// is_superset checks if the given Self is a superset of the target, i.e.
    /// all the squares in the given Self are also present in the target.
    fn is_superset(self, other: Self) -> bool {
        other.is_subset(self)
    }

    /// is_empty checks if the target Self is empty.
    fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    /// cardinality returns the number of Squares present in the Self.
    fn cardinality(self) -> usize {
        self.count_ones() as usize
    }

    /// contains checks if the Self contains the given Self::Square.
    fn contains(self, square: Self::Square) -> bool {
        self & Self::from(square) != Self::EMPTY
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

    /// insert puts the given Self::Square into the Self.
    fn insert(&mut self, square: Self::Square) {
        *self = *self | Self::from(square)
    }

    /// remove removes the given Self::Square from the Self.
    fn remove(&mut self, square: Self::Square) {
        *self = *self & !Self::from(square)
    }

    /// pop_lsb pops the least significant Self::Square from the Self, i.e. it
    /// removes the lsb from the Self and returns its value.
    fn pop_lsb(&mut self) -> Option<Self::Square> {
        let lsb = self.lsb();
        let copy = *self;
        *self = copy & (copy - 1);

        lsb
    }

    /// pop_msb pops the most significant Self::Square from the Self i.e. it
    /// removes the msb from the Self and returns its value.
    fn pop_msb(&mut self) -> Option<Self::Square> {
        let msb = self.msb();
        if let Some(msb) = msb {
            *self = *self ^ Self::from(msb);
        }

        msb
    }

    /// get_lsb returns the least significant Self::Square from the Self.
    fn lsb(self) -> Option<Self::Square> {
        let sq = self.trailing_zeros() as usize;
        if sq < Self::Square::N {
            Some(unsafe { Self::Square::unsafe_from(sq) })
        } else {
            None
        }
    }

    /// get_msb returns the most significant Self::Square from the Self.
    fn msb(self) -> Option<Self::Square> {
        let sq = 63 - self.leading_zeros() as usize;
        if sq < Self::Square::N {
            Some(unsafe { Self::Square::unsafe_from(sq) })
        } else {
            None
        }
    }

    /// file returns a Self containing all the squares from the given <Self::Square as Square>::File.
    fn file(file: <Self::Square as SquareType>::File) -> Self {
        Self::FIRST_FILE << file.into() as usize
    }

    /// rank returns a Self containing all the squares from the given Self::Square::Rank.
    fn rank(rank: <Self::Square as SquareType>::Rank) -> Self {
        Self::FIRST_RANK << (<Self::Square as SquareType>::File::N * rank.into() as usize)
    }

    fn count_ones(&self) -> u32;
    fn leading_zeros(&self) -> u32;
    fn trailing_zeros(&self) -> u32;
}
