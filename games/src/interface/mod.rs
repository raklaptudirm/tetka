use std::fmt::{Debug, Display};
use std::str::FromStr;

use strum::IntoEnumIterator;
use thiserror::Error;

mod bitboard;
mod hash;
mod r#move;
mod piece;
mod position;
mod square;

pub use bitboard::*;
pub use hash::*;
pub use piece::*;
pub use position::*;
pub use r#move::*;
pub use square::*;

pub type BitBoard<P> = <P as PositionType>::BitBoard;
pub type Square<P> = <BitBoard<P> as BitBoardType>::Square;
pub type File<P> = <Square<P> as SquareType>::File;
pub type Rank<P> = <Square<P> as SquareType>::Rank;

pub type ColoredPiece<P> = <P as PositionType>::ColoredPiece;
pub type Piece<P> = <ColoredPiece<P> as ColoredPieceType>::Piece;
pub type Color<P> = <ColoredPiece<P> as ColoredPieceType>::Color;

pub type Move<P> = <P as PositionType>::Move;

/// RepresentableType is a basic trait which is implemented by enums with both a
/// binary and string representation and backed by an integer.
pub trait RepresentableType<B: Into<usize>>:
    Copy + Eq + FromStr + Display + Into<B> + TryFrom<B, Error: Debug> + IntoEnumIterator
{
    /// N is the number of specializations of the enum.
    const N: usize;

    /// unsafe_from unsafely converts the given number into Self.
    /// # Safety
    /// `unsafe_from` assumes that the target type can represent the provided
    /// number, i.e. the number has a valid representation in the target type.
    /// The function comes with a debug check for the same, and failure to
    /// uphold this invariant will result in undefined behavior.
    #[must_use]
    unsafe fn unsafe_from<T: Copy + Into<usize>>(number: T) -> Self {
        debug_assert!(number.into() < Self::N);
        std::mem::transmute_copy(&number)
    }
}

#[derive(Error, Debug)]
pub enum TypeParseError {
    #[error("invalid {0} identifier string")]
    StrError(String),
    #[error("invalid integer representation for {0}")]
    RangeError(String),
}

macro_rules! representable_type {
    ($(#[doc = $doc:expr])* enum $type:tt: $base:tt { $($variant:tt $repr:expr,)* }) => {
        $(#[doc = $doc])*
        #[derive(Copy, Clone, PartialEq, Eq, Debug, strum_macros::EnumIter)]
        #[repr($base)]
        pub enum $type { $($variant,)* }

        impl RepresentableType<$base> for $type {
            const N: usize = 0 $(+ representable_type!(@__puke_1 $variant))*;
        }

        impl From<$type> for $base {
            #[must_use]
            fn from(value: $type) -> Self {
                value as $base
            }
        }

        impl TryFrom<$base> for $type {
            type Error = $crate::interface::TypeParseError;

            fn try_from(value: $base) -> Result<Self, Self::Error> {
                if value as usize >= Self::N {
                    Err($crate::interface::TypeParseError::RangeError(stringify!($type).to_string()))
                } else {
                    Ok(unsafe { Self::unsafe_from(value) })
                }
            }
        }

        impl FromStr for $type {
            type Err = $crate::interface::TypeParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($repr => Ok(Self::$variant),)*
                    _ => Err($crate::interface::TypeParseError::StrError(stringify!($type).to_string())),
                }
            }
        }

        impl fmt::Display for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $(Self::$variant => write!(f, "{}", $repr),)*
                }
            }
        }
    };

    (@__puke_1 $t:tt) => { 1 };
}

pub(crate) use representable_type;

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
    ($(#[doc = $doc:expr])* struct $name:tt : $typ:tt {
        Square = $sq:tt;
        Empty = $empty:expr;
        Universe = $universe:expr;
        FirstFile = $first_file:expr;
        FirstRank = $first_rank:expr;
    } ) => {
        $(#[doc = $doc])*
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
                let lsb = if self.is_empty() {
                    None
                } else {
                    let sq = <Self as Into<<Self as BitBoardType>::Base>>::into(*self).trailing_zeros() as usize;
                    Some(unsafe { <Self as BitBoardType>::Square::unsafe_from(sq) })
                };

                if !self.is_empty() {
                    let copy = *self;
                    *self = copy & (copy - 1);
                }

                lsb
            }
        }

        impl std::ops::Sub<usize> for $name {
            type Output = Self;

            #[must_use]
            fn sub(self, rhs: usize) -> Self::Output {
                Self(self.0 - rhs as u64)
            }
        }

        impl From<$typ> for $name {
            #[must_use]
            fn from(num: $typ) -> Self {
                Self(num)
            }
        }

        impl From<$name> for $typ {
            #[must_use]
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl From<$sq> for $name {
            #[must_use]
            fn from(square: $sq) -> Self {
                Self(1 << square as u64)
            }
        }

        impl std::ops::Not for $name {
            type Output = Self;

            /// Returns the complementary BitBoard of `self`.
            #[must_use]
            fn not(self) -> Self::Output {
                // ! will set the unused bits so remove them with an &.
                Self(!self.0) & <Self as $crate::interface::BitBoardType>::UNIVERSE
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl std::ops::Sub for $name {
            type Output = Self;

            /// Returns the difference of `self` and `rhs` as a new BitBoard.
            #[must_use]
            fn sub(self, rhs: Self) -> Self::Output {
                self & !rhs
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl std::ops::BitOr<$sq> for $name {
            type Output = Self;

            /// Returns the union of `self` and `rhs` as a new BitBoard.
            #[must_use]
            fn bitor(self, rhs: $sq) -> Self::Output {
                self | Self::from(rhs)
            }
        }

        impl std::ops::Sub<$sq> for $name {
            type Output = Self;

            /// Returns the BitBoard obtained on removing `rhs` from `self`.
            #[must_use]
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

/// PositionParseErr represents an error encountered while parsing
/// the given FEN position field into a valid Position.
#[derive(Error, Debug)]
pub enum PiecePlacementParseError {
    #[error("a jump value was too long and overshot")]
    JumpTooLong,

    #[error("invalid piece identifier '{0}'")]
    InvalidPieceIdent(char),
    #[error("insufficient data to fill the entire {0} file")]
    FileDataIncomplete(String),
    #[error("expected {0} ranks, found more")]
    TooManyRanks(usize),
}

pub(crate) fn parse_piece_placement<T: PositionType>(
    position: &mut T,
    fen_fragment: &str,
) -> Result<(), PiecePlacementParseError> {
    for sq in <<<T as PositionType>::BitBoard as BitBoardType>::Square as IntoEnumIterator>::iter()
    {
        position.remove(sq);
    }

    // Spilt the position spec by the Ranks which are separated by '/'.
    let ranks: Vec<&str> = fen_fragment.split('/').collect();

    let first_file = File::<T>::iter().next().unwrap();

    let mut file = Ok(first_file);
    let mut rank = Ok(Rank::<T>::iter().last().unwrap());

    // Iterate over the Ranks in the string spec.
    for rank_data in ranks {
        // Rank pointer ran out, but data carried on.
        if rank.is_err() {
            return Err(PiecePlacementParseError::TooManyRanks(<<<T as PositionType>::BitBoard as BitBoardType>::Square as SquareType>::Rank::iter().len()));
        }

        // Iterate over the Square specs in the Rank spec.
        for data in rank_data.chars() {
            // Check if a jump spec was too big and we landed on an invalid File.
            if file.is_err() {
                return Err(PiecePlacementParseError::JumpTooLong);
            }

            let file_value = *file.as_ref().unwrap();
            let rank_value = *rank.as_ref().unwrap();
            let square = <Square<T>>::new(file_value, rank_value);
            match data {
                // Numbers represent jump specs to jump over empty squares.
                '1'..='8' => {
                    file = <File<T>>::try_from(file_value.into() + data as u8 - b'1');
                    if file.is_err() {
                        return Err(PiecePlacementParseError::JumpTooLong);
                    }
                }

                _ => match <ColoredPiece<T>>::from_str(&data.to_string()) {
                    Ok(piece) => position.insert(square, piece),
                    Err(_) => return Err(PiecePlacementParseError::InvalidPieceIdent(data)),
                },
            }

            // On to the next Square spec in the Rank spec.
            file = <File<T>>::try_from(file.unwrap().into() + 1);
        }

        // After rank data runs out, file pointer should be
        // at the last file, i.e, rank is completely filled.
        if let Ok(file) = file {
            return Err(PiecePlacementParseError::FileDataIncomplete(
                file.to_string(),
            ));
        }

        // Switch rank pointer and reset file pointer.
        rank = <Rank<T>>::try_from((rank.unwrap().into()).wrapping_sub(1));
        file = Ok(first_file);
    }

    Ok(())
}
