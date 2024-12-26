//! A common interface for game logic.
//!
//! The main type when working with the logic for a game is the [`PositionType`]
//! trait, which defines the common interface for the position representations
//! of games. All the details regarding a game can be found as a child of the
//! [`PositionType`] trait, see its documentation for more information.

use std::fmt::{Debug, Display};
use std::num::ParseIntError;
use std::str::FromStr;

use strum::IntoEnumIterator;
use thiserror::Error;

mod bitboard;
mod hash;
mod r#move;
mod piece;
mod position;
mod set;
mod square;

pub use bitboard::*;
pub use hash::*;
pub use piece::*;
pub use position::*;
pub use r#move::*;
pub use set::*;
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
    Copy
    + Eq
    + FromStr
    + Display
    + Into<B>
    + TryFrom<B, Error: Debug>
    + IntoEnumIterator
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

macro_rules! game_details {
    (
        Files: $($file_variant:ident),* ;
        Ranks: $($rank_number:literal $rank_variant:ident),* ;

        Pieces: $($piece_variant:ident $piece_repr:literal),*;
                $($other_variant:ident $other_repr:literal),*;
        Colors: $color_1:ident $color_1_repr:literal ($($piece_1_repr:literal),*),
                $color_2:ident $color_2_repr:literal ($($piece_2_repr:literal),*);
    ) => {};

    (
        @bitboard_less
        Files: $($file_variant:ident),* ;
        Ranks: $($rank_number:literal $rank_variant:ident),* ;

        Pieces: $($piece_variant:ident $piece_repr:literal),*;
                $($other_variant:ident $other_repr:literal),*;
        Colors: $color_1:ident $color_1_repr:literal ($($piece_1_repr:literal),*),
                $color_2:ident $color_2_repr:literal ($($piece_2_repr:literal),*);
    ) => {
        game_details!(
            @squares
            Files: $($file_variant),* ;
            Ranks: $($rank_number $rank_variant),* ;
        );

        game_details!(
            @pieces
            Pieces: $($piece_variant $piece_repr),*;
                    $($other_variant $other_repr),*;
            Colors: $color_1 $color_1_repr ($($piece_1_repr),*),
                    $color_2 $color_2_repr ($($piece_2_repr),*);
        );
    };

    (
        @pieces
        Pieces: $($piece_variant:ident $piece_repr:literal),*;
                $($other_variant:ident $other_repr:literal),*;
        Colors: $color_1:ident $color_1_repr:literal ($($piece_1_repr:literal),*),
                $color_2:ident $color_2_repr:literal ($($piece_2_repr:literal),*);
    ) => {
        game_details!(
            @color $color_1 $color_1_repr; $color_2 $color_2_repr;
        );

        $crate::interface::representable_type!(
            enum Piece: u8 {
                $($piece_variant $piece_repr,)*
                $($other_variant $other_repr,)*
            }
        );

        paste::paste!(
            $crate::interface::representable_type!(
                enum ColoredPiece: u8 {
                    $([< $color_1 $piece_variant >] $piece_1_repr,)*
                    $([< $color_2 $piece_variant >] $piece_2_repr,)*
                    $($other_variant $other_repr,)*
                }
            );
        );

        impl $crate::interface::ColoredPieceType for ColoredPiece {
            type Piece = Piece;
            type Color = Color;

            fn piece(self) -> Self::Piece {
                paste::paste!(
                    match self {
                        $(
                            Self:: [< $color_1 $piece_variant >] |
                            Self:: [< $color_2 $piece_variant >]
                                => Self::Piece::$piece_variant,
                        )*

                        $(Self:: $other_variant => Self::Piece::$other_variant)*
                    }
                )
            }

            #[allow(unreachable_patterns)]
            fn color(self) -> Self::Color {
                paste::paste!(
                    match self {
                        $(Self:: [< $color_1 $piece_variant >])|* => Self::Color::$color_1,
                        $(Self:: [< $color_2 $piece_variant >])|* => Self::Color::$color_2,
                        _ => panic!("ColoredPiece::color() called on uncolored piece")
                    }
                )
            }
        }
    };

    (
        @color
        $first:tt $first_repr:expr;
        $second:tt $second_repr:expr;
    ) => {
        crate::interface::representable_type! {
            enum Color: u8 {
                $first $first_repr, $second $second_repr,
            }
        }

        impl std::ops::Not for Color {
            type Output = Self;

            fn not(self) -> Self::Output {
                unsafe { <Self as $crate::interface::RepresentableType<u8>>::unsafe_from(self as usize ^ 1) }
            }
        }

        impl crate::interface::ColorType for Color {
            fn first() -> Self {
                Self::$first
            }
        }
    };

    (
        @squares
        Files: $($file_variant:ident),* ;
        Ranks: $($rank_number:literal $rank_variant:ident),* ;
    ) => {
        game_details!(
            @file_rank_product $($rank_number),*;$($file_variant),*
        );

        impl $crate::interface::SquareType for Square {
            type File = File;
            type Rank = Rank;
        }

        $crate::interface::representable_type!(
            @no_display [[File] [u8]] [$([$file_variant])*]
        );

        impl std::str::FromStr for File {
            type Err = $crate::interface::TypeParseError;

            #[allow(clippy::char_lit_as_u8)]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.len() != 1 {
                    Err($crate::interface::TypeParseError::StrError(
                        stringify!(File).to_string()
                    ))
                } else {
                    unsafe {
                        let file_idx = s.chars().next().unwrap_unchecked() as u8 - 'a' as u8;
                        if file_idx < <File as $crate::interface::RepresentableType<u8>>::N as u8 {
                                Ok(<File as $crate::interface::RepresentableType<u8>>::unsafe_from(file_idx))
                        } else {
                            Err($crate::interface::TypeParseError::StrError(
                                stringify!(File).to_string()
                            ))
                        }
                    }
                }
            }
        }

        impl std::fmt::Display for File {
            #[allow(clippy::char_lit_as_u8)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", (*self as u8 + 'a' as u8) as char)
            }
        }

        $crate::interface::representable_type!(
            @no_display [[Rank] [u8]] [$([$rank_variant])*]
        );

        impl std::str::FromStr for Rank {
            type Err = $crate::interface::TypeParseError;

            #[allow(clippy::char_lit_as_u8)]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.len() != 1 {
                    Err($crate::interface::TypeParseError::StrError(
                        stringify!(Rank).to_string()
                    ))
                } else {
                    unsafe {
                        let rank_idx = s.chars().next().unwrap_unchecked() as u8 - '1' as u8;
                        if rank_idx < <Rank as $crate::interface::RepresentableType<u8>>::N as u8 {
                                Ok(<Rank as $crate::interface::RepresentableType<u8>>::unsafe_from(rank_idx))
                        } else {
                            Err($crate::interface::TypeParseError::StrError(
                                stringify!(Rank).to_string()
                            ))
                        }
                    }
                }
            }
        }

        impl std::fmt::Display for Rank {
            #[allow(clippy::char_lit_as_u8)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", (*self as u8 + '1' as u8) as char)
            }
        }
    };

    (@file_rank_product $($e1:expr),* ; $($e2:expr),*) => {
        $crate::interface::representable_type!(@cartesian Square, u8; [$([$e1])*][$([$e2])*]);

        impl std::str::FromStr for Square {
            type Err = $crate::interface::TypeParseError;

            #[allow(clippy::char_lit_as_u8)]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.len() != 2 {
                    Err($crate::interface::TypeParseError::StrError(
                        stringify!(File).to_string()
                    ))
                } else {
                    let file = File::from_str(&s[..1]);
                    let rank = Rank::from_str(&s[1..]);

                    if let (Ok(file), Ok(rank)) = (file, rank) {
                        Ok($crate::interface::SquareType::new(file, rank))
                    } else {
                        Err($crate::interface::TypeParseError::StrError(
                            stringify!(File).to_string()
                        ))
                    }
                }
            }
        }

        impl std::fmt::Display for Square {
            #[allow(clippy::char_lit_as_u8)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{}",
                    $crate::interface::SquareType::file(*self),
                    $crate::interface::SquareType::rank(*self),
                )
            }
        }
    };
}
pub(crate) use game_details;

macro_rules! representable_type {
    ($(#[doc = $doc:expr])* enum $type:tt: $base:tt {
        $($variant:tt $repr:expr,)*
    }) => {
        $crate::interface::representable_type! {
            @no_display [[$type] [$base]] [$([$variant])*]
        }

        impl std::str::FromStr for $type {
            type Err = $crate::interface::TypeParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($repr => Ok(Self::$variant),)*
                    _ => Err(
                        $crate::interface::TypeParseError::StrError(
                            stringify!($type).to_string()
                        )
                    ),
                }
            }
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $(Self::$variant => write!(f, "{}", $repr),)*
                }
            }
        }
    };

    (@no_display [[$type:tt] [$base:tt]] [$([$variant:tt])*]) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug, strum_macros::EnumIter)]
        #[repr($base)]
        pub enum $type { $($variant,)* }

        impl $crate::interface::RepresentableType<$base> for $type {
            const N: usize = 0 $(+ $crate::interface::representable_type!(@puke_1 $variant))*;
        }

        $crate::interface::representable_type!(@impl $type $base);
    };

    (@cartesian $type:ident, $base:tt; [$([$e1:expr])*]$e2:tt) => {
        $crate::interface::representable_type!(@cartesian_helper $type, $base; $([[$e1]$e2])*);
        $crate::interface::representable_type!(@impl $type $base);
    };

    // [[1] [[A] [B] [C]]] [[2] [[A] [B] [C]]] -> [A1, B1, ...]
    (@cartesian_helper $type:ident, $base:tt; $([[$e1:expr][$([$e2:expr])*]])*) => {
        paste::paste! {
            #[derive(Copy, Clone, PartialEq, Eq, Debug, strum_macros::EnumIter)]
            #[repr($base)]
            pub enum $type {
                $($([<$e2 $e1>]),*),*
            }
        }

        impl $crate::interface::RepresentableType<$base> for $type {
            const N: usize = 0 $($(+ $crate::interface::representable_type!(@puke_1 $e1 $e2))*)*;
        }
    };

    (@impl $type:ident $base:tt) => {
        impl From<$type> for $base {
            #[must_use]
            fn from(value: $type) -> Self {
                value as $base
            }
        }

        impl TryFrom<$base> for $type {
            type Error = $crate::interface::TypeParseError;

            fn try_from(value: $base) -> Result<Self, Self::Error> {
                if value as usize >= <Self as $crate::interface::RepresentableType<$base>>::N {
                    Err(
                        $crate::interface::TypeParseError::RangeError(
                            "stringify!($type).to_string()".to_string()
                        )
                    )
                } else {
                    Ok(unsafe { <Self as $crate::interface::RepresentableType<$base>>::unsafe_from(value) })
                }
            }
        }
    };

    (@puke_1 $($t:tt)*) => { 1 };
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
macro_rules! set_type {
    ($(#[doc = $doc:expr])* $name:tt<$sq:tt>: $typ:tt) => {
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
        )]
        pub struct $name(pub $typ);

        impl crate::interface::SetType<$typ, $sq> for $name {
            const EMPTY: Self = Self(0);
            const UNIVERSE: Self = Self(match (1 as $typ).checked_shl(<$sq as $crate::interface::RepresentableType<_>>::N as u32) {
                Some(universe) => universe.wrapping_sub(1),
                None => (-1i8) as $typ,
            });
        }

        impl Iterator for $name {
            type Item = $sq;

            /// next pops the next Square from the BitBoard and returns it.
            fn next(&mut self) -> Option<Self::Item> {
                let lsb = if crate::interface::SetType::<$typ, $sq>::is_empty(*self) {
                    None
                } else {
                    let sq = <Self as Into<$typ>>::into(
                        *self,
                    )
                    .trailing_zeros() as usize;
                    Some(unsafe {
                        <$sq as $crate::interface::RepresentableType<u8>>::unsafe_from(sq)
                    })
                };

                if !crate::interface::SetType::<$typ, $sq>::is_empty(*self) {
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
                Self(self.0 - rhs as $typ)
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
                Self($typ::from(1u8) << $typ::from(u8::from(square)))
            }
        }

        impl std::ops::Not for $name {
            type Output = Self;

            /// Returns the complementary BitBoard of `self`.
            #[must_use]
            fn not(self) -> Self::Output {
                // ! will set the unused bits so remove them with an &.
                Self(!self.0)
                    & <Self as crate::interface::SetType<$typ, $sq>>::UNIVERSE
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
        impl std::ops::SubAssign for $name {
            /// Returns the difference of `self` and `rhs` as a new BitBoard.
            fn sub_assign(&mut self, rhs: Self) {
                *self &= !rhs
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
    };
}

pub(crate) use set_type;

macro_rules! bitboard_type {
    ($(#[doc = $doc:expr])* struct $name:tt : $typ:tt {
        Square = $sq:tt;
        Empty = $empty:expr;
        Universe = $universe:expr;
        FirstFile = $first_file:expr;
        FirstRank = $first_rank:expr;
    }) => {
        crate::interface::set_type!($name<$sq>: $typ);

        impl crate::interface::BitBoardType for $name {
            type Base = $typ;
            type Square = $sq;

            const FIRST_FILE: Self = $first_file;
            const FIRST_RANK: Self = $first_rank;
        }

        // Display a bitboard as ASCII art with 0s and 1s.
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut string_rep = String::from("");
                for rank in
                    <
                        <$sq as crate::interface::SquareType>::Rank
                            as strum::IntoEnumIterator
                    >::iter().rev()
                {
                    for file in
                        <
                            <$sq as crate::interface::SquareType>::File
                                as strum::IntoEnumIterator
                        >::iter()
                    {
                        let square = <$sq as crate::interface::SquareType>
                            ::new(file, rank);
                        string_rep += if crate::interface::SetType::<$typ, $sq>::contains(*self, square) {
                            "1 "
                        } else {
                            "0 "
                        };
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
    for sq in <<<T as PositionType>::BitBoard as BitBoardType>::Square
        as IntoEnumIterator>::iter()
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
            return Err(PiecePlacementParseError::TooManyRanks(
                <<<T as PositionType>::BitBoard as BitBoardType>::Square
                    as SquareType>::Rank::iter().len()
            ));
        }

        // Iterate over the Square specs in the Rank spec.
        for data in rank_data.chars() {
            // Check if a jump was too big and we landed on an invalid File.
            if file.is_err() {
                return Err(PiecePlacementParseError::JumpTooLong);
            }

            let file_value = *file.as_ref().unwrap();
            let rank_value = *rank.as_ref().unwrap();
            let square = <Square<T>>::new(file_value, rank_value);
            match data {
                // Numbers represent jump specs to jump over empty squares.
                '1'..='8' => {
                    file = <File<T>>::try_from(
                        file_value.into() + data as u8 - b'1',
                    );
                    if file.is_err() {
                        return Err(PiecePlacementParseError::JumpTooLong);
                    }
                }

                _ => match <ColoredPiece<T>>::from_str(&data.to_string()) {
                    Ok(piece) => position.insert(square, piece),
                    Err(_) => {
                        return Err(
                            PiecePlacementParseError::InvalidPieceIdent(data),
                        )
                    }
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

pub(crate) fn parse_ply_count<C: ColorType>(
    fmc: &str,
    stm: C,
) -> Result<u16, ParseIntError> {
    if stm == C::first() {
        Ok(fmc.parse::<u16>()? * 2 - 1)
    } else {
        Ok(fmc.parse::<u16>()? * 2 - 2)
    }
}
