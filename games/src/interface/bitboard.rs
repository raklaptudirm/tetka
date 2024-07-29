use std::ops::BitOr;

use super::{RepresentableType, SquareType};

pub trait BitBoardType:
    Sized
    + Copy
    + From<Self::Square>
    + From<u64>
    + Into<u64>
    + BitOr<Self>
    + From<<Self as BitOr<Self>>::Output>
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
        self.into() & other.into() == Self::EMPTY.into()
    }

    /// is_subset checks if the given Self is a subset of the target, i.e.
    /// all the squares in the target are also present in the given Self.
    fn is_subset(self, other: Self) -> bool {
        other.into() & !self.into() == Self::EMPTY.into()
    }

    /// is_superset checks if the given Self is a superset of the target, i.e.
    /// all the squares in the given Self are also present in the target.
    fn is_superset(self, other: Self) -> bool {
        other.is_subset(self)
    }

    /// is_empty checks if the target Self is empty.
    fn is_empty(self) -> bool {
        self.into() == Self::EMPTY.into()
    }

    /// cardinality returns the number of Squares present in the Self.
    fn cardinality(self) -> usize {
        self.into().count_ones() as usize
    }

    /// contains checks if the Self contains the given Self::Square.
    fn contains(self, square: Self::Square) -> bool {
        self.into() & (1 << square.into()) != Self::EMPTY.into()
    }

    /// north returns a new Self with all the squares shifted to the north.
    fn north(self) -> Self {
        Self::from((self.into() << <Self::Square as SquareType>::File::N) & Self::UNIVERSE.into())
    }

    /// south returns a new Self with all the squares shifted to the south.
    fn south(self) -> Self {
        Self::from(self.into() >> <Self::Square as SquareType>::File::N)
    }

    /// east returns a new Self with all the squares shifted to the east.
    fn east(self) -> Self {
        Self::from(
            (self.into() << 1)
                & (Self::UNIVERSE.into()
                    ^ Self::file(<Self::Square as SquareType>::File::from(0)).into()),
        )
    }

    /// west returns a new Self with all the squares shifted to the west.
    fn west(self) -> Self {
        Self::from(
            (self.into() >> 1)
                & (Self::UNIVERSE.into()
                    ^ Self::file(<Self::Square as SquareType>::File::from(
                        <Self::Square as SquareType>::File::N as u8 - 1,
                    ))
                    .into()),
        )
    }

    /// insert puts the given Self::Square into the Self.
    fn insert(&mut self, square: Self::Square) {
        *self =
            Self::from(<Self as std::convert::Into<u64>>::into(*self) | Self::from(square).into())
    }

    /// remove removes the given Self::Square from the Self.
    fn remove(&mut self, square: Self::Square) {
        *self =
            Self::from(<Self as std::convert::Into<u64>>::into(*self) & !Self::from(square).into())
    }

    /// pop_lsb pops the least significant Self::Square from the Self, i.e. it
    /// removes the lsb from the Self and returns its value.
    fn pop_lsb(&mut self) -> Option<Self::Square> {
        let lsb = self.lsb();
        let copy = <Self as Into<u64>>::into(*self);
        *self = <Self as From<u64>>::from(copy & (copy - 1));

        lsb
    }

    /// pop_msb pops the most significant Self::Square from the Self i.e. it
    /// removes the msb from the Self and returns its value.
    fn pop_msb(&mut self) -> Option<Self::Square> {
        let msb = self.msb();
        if let Some(msb) = msb {
            *self = Self::from(<Self as Into<u64>>::into(*self) ^ Self::from(msb).into());
        }

        msb
    }

    /// get_lsb returns the least significant Self::Square from the Self.
    fn lsb(self) -> Option<Self::Square> {
        let sq = self.into().trailing_zeros() as usize;
        if sq < Self::Square::N {
            Some(unsafe { Self::Square::unsafe_from(sq) })
        } else {
            None
        }
    }

    /// get_msb returns the most significant Self::Square from the Self.
    fn msb(self) -> Option<Self::Square> {
        let sq = 63 - self.into().leading_zeros() as usize;
        if sq < Self::Square::N {
            Some(unsafe { Self::Square::unsafe_from(sq) })
        } else {
            None
        }
    }

    fn singles(self) -> Self {
        let bar: Self = ((self | self.east()).into() | self.west()).into();
        ((bar | bar.north()).into() | bar.south()).into()
    }

    /// file returns a Self containing all the squares from the given <Self::Square as Square>::File.
    fn file(file: <Self::Square as SquareType>::File) -> Self {
        Self::from(Self::FIRST_FILE.into() << file.into())
    }

    /// rank returns a Self containing all the squares from the given Self::Square::Rank.
    fn rank(rank: <Self::Square as SquareType>::Rank) -> Self {
        Self::from(
            Self::FIRST_RANK.into()
                << (<Self::Square as SquareType>::File::N * rank.into() as usize),
        )
    }
}
