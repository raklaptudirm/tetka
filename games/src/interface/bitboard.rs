use num_traits::int::PrimInt;
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub};

use super::{RepresentableType, SquareType};

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

    /// is_disjoint checks if the two Selfs are disjoint, i.e. don't have
    /// any squares in common among themselves.
    fn is_disjoint(self, other: Self) -> bool {
        (self & other).is_empty()
    }

    /// is_subset checks if the given Self is a subset of the target, i.e.
    /// all the squares in the target are also present in the given Self.
    fn is_subset(self, other: Self) -> bool {
        (other & !self).is_empty()
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
    fn len(self) -> usize {
        self.into().count_ones() as usize
    }

    /// contains checks if the Self contains the given Self::Square.
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

        if !self.is_empty() {
            let copy = *self;
            *self = copy & (copy - 1);
        }

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

    /// Clears the BitBoard, removing all elements.
    fn clear(&mut self) {
        *self = Self::EMPTY
    }

    /// get_lsb returns the least significant Self::Square from the Self.
    fn lsb(self) -> Option<Self::Square> {
        if self.is_empty() {
            None
        } else {
            let sq = self.into().trailing_zeros() as usize;
            Some(unsafe { Self::Square::unsafe_from(sq) })
        }
    }

    /// get_msb returns the most significant Self::Square from the Self.
    fn msb(self) -> Option<Self::Square> {
        if self.is_empty() {
            None
        } else {
            let sq = 63 - self.into().leading_zeros() as usize;
            Some(unsafe { Self::Square::unsafe_from(sq) })
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
    /// The elements are visited in ascending order.
    fn retain<F: FnMut(Self::Square) -> bool>(&mut self, mut f: F) {
        for sq in *self {
            if !f(sq) {
                self.remove(sq)
            }
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
}

macro_rules! bitboard_type {
    ($name:tt : $typ:tt { Square = $sq:tt; Empty = $empty:expr; Universe = $universe:expr; FirstFile = $first_file:expr; FirstRank = $first_rank:expr; } ) => {
        #[derive(
            Copy,
            Clone,
            PartialEq,
            Eq,
            num_derive::FromPrimitive,
            derive_more::BitOr,
            derive_more::BitAnd,
            derive_more::BitXor,
            derive_more::Shl,
            derive_more::Shr,
            derive_more::BitAndAssign,
            derive_more::BitOrAssign,
            derive_more::BitXorAssign,
            derive_more::ShlAssign,
            derive_more::ShrAssign,
            derive_more::SubAssign,
        )]
        pub struct $name(pub $typ);

        impl $crate::interface::BitBoardType for $name {
            type Base = $typ;
            type Square = $sq;

            const EMPTY: Self = $empty;
            const UNIVERSE: Self = $universe;
            const FIRST_FILE: Self = $first_file;
            const FIRST_RANK: Self = $first_rank;
        }

        // Iterator trait allows $name to be used in a for loop.
        impl Iterator for $name {
            type Item = $sq;

            fn next(&mut self) -> Option<Self::Item> {
                $crate::interface::BitBoardType::pop_lsb(self)
            }
        }

        impl std::ops::Sub<usize> for $name {
            type Output = Self;

            fn sub(self, rhs: usize) -> Self::Output {
                Self(self.0 - rhs as u64)
            }
        }

        impl From<$typ> for $name {
            fn from(num: $typ) -> Self {
                Self(num)
            }
        }

        impl From<$name> for $typ {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        // From trait allows a square to be converted into it's $name representation.
        impl From<$sq> for $name {
            fn from(square: $sq) -> Self {
                Self(1 << square as u64)
            }
        }

        // Not(!)/Complement operation implementation for $name.
        impl std::ops::Not for $name {
            type Output = Self;

            fn not(self) -> Self::Output {
                // ! will set the unused bits so remove them with an &.
                Self(!self.0) & <Self as $crate::interface::BitBoardType>::UNIVERSE
            }
        }

        // Implementation of subtraction(removal) of BitBoards.
        #[allow(clippy::suspicious_arithmetic_impl)]
        impl std::ops::Sub for $name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                self & !rhs
            }
        }

        // Implementation of |(or)/set-union of a $name with a $sq.
        #[allow(clippy::suspicious_arithmetic_impl)]
        impl std::ops::BitOr<$sq> for $name {
            type Output = Self;

            fn bitor(self, rhs: $sq) -> Self::Output {
                self | Self::from(rhs)
            }
        }

        // Implementation of -(subtraction)/set-removal of a $name with a $sq.
        impl std::ops::Sub<$sq> for $name {
            type Output = Self;

            fn sub(self, rhs: $sq) -> Self::Output {
                self & !Self::from(rhs)
            }
        }

        // Display a bitboard as ASCII art with 0s and 1s.
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut string_rep = String::from("");
                for rank in <<$sq as $crate::interface::SquareType>::Rank as strum::IntoEnumIterator>::iter().rev() {
                    for file in <<$sq as $crate::interface::SquareType>::File as strum::IntoEnumIterator>::iter() {
                        let square = <$sq as $crate::interface::SquareType>::new(file, rank);
                        string_rep += if $crate::interface::BitBoardType::contains(*self, square) { "1 " } else { "0 " };
                    }

                    string_rep += "\n";
                }

                write!(f, "{string_rep}")
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self)
            }
        }
    };
}

pub(crate) use bitboard_type;
