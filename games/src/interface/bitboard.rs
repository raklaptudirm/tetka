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

/// bitboard_type generates a new BitBoard with the given type as its base
/// representation. The base type must implement num_traits::int::PrimInt so
/// that the BitBoardType trait can be implemented.
///
/// # Examples
///
/// ```
/// bitboard_type! {
///     BitBoardTypeName: u64 {
///         Square = OurSquareType;
///         Empty = OurEmptyBitBoard;
///         Universe = OurUniverseBitBoard;
///         FirstFile = OurFirstFileBitBoard;
///         FirstRank = OurFirstRankBitBoard;
///     }
/// }
/// ```
macro_rules! bitboard_type {
    ($name:tt : $typ:tt {
        Square = $sq:tt;
        Empty = $empty:expr;
        Universe = $universe:expr;
        FirstFile = $first_file:expr;
        FirstRank = $first_rank:expr;
    } ) => {
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

        impl Iterator for $name {
            type Item = $sq;

            /// next pops the next Square from the BitBoard and returns it.
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

        impl From<$sq> for $name {
            fn from(square: $sq) -> Self {
                Self(1 << square as u64)
            }
        }

        impl std::ops::Not for $name {
            type Output = Self;

            /// Returns the complementary BitBoard of `self`.
            fn not(self) -> Self::Output {
                // ! will set the unused bits so remove them with an &.
                Self(!self.0) & <Self as $crate::interface::BitBoardType>::UNIVERSE
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl std::ops::Sub for $name {
            type Output = Self;

            /// Returns the difference of `self` and `rhs` as a new BitBoard.
            fn sub(self, rhs: Self) -> Self::Output {
                self & !rhs
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl std::ops::BitOr<$sq> for $name {
            type Output = Self;

            /// Returns the union of `self` and `rhs` as a new BitBoard.
            fn bitor(self, rhs: $sq) -> Self::Output {
                self | Self::from(rhs)
            }
        }

        impl std::ops::Sub<$sq> for $name {
            type Output = Self;

            /// Returns the BitBoard obtained on removing `rhs` from `self`.
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
