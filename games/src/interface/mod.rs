//! A common interface for game logic.
//!
//! The main type when working with the logic for a game is the [`PositionType`]
//! trait, which defines the common interface for the position representations
//! of games. All the details regarding a game can be found as a child of the
//! [`PositionType`] trait, see its documentation for more information.

use std::fmt::{Debug, Display};
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

pub(crate) mod parse;

pub use bitboard::*;
pub use hash::*;
pub use piece::*;
pub use position::*;
pub use r#move::*;
pub use set::*;
pub use square::*;

pub type Square<P> = <P as PositionType>::Square;
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
    #[error("invalid string representation for {0}")]
    StrError(String),
    #[error("invalid integer representation for {0}")]
    RangeError(String),
}

// The game_details macro in an internal macro which provides a bunch of
// utility sub-macros which can be used to generate a ton of boilerplate code
// while developing a game backend for tetka-games.
//
// Sub-macros can be called by prefixing the macro body with a valid decorator.
// Accepted ones include @bitboard_less, @squares, @pieces, @color, etc.
//
// The undecorated macro essentially does the work of all the sub-macros put
// together. Use the undecorated macro unless more precise control is needed.
macro_rules! game_details {
    (
        Files: $($file_variant:ident),* ;
        Ranks: $($rank_number:literal $rank_variant:ident),* ;

        Pieces: $($piece_variant:ident $piece_repr:literal),*;
                $($other_variant:ident $other_repr:literal),*;
        Colors: $color_1:ident $color_1_repr:literal ($($piece_1_repr:literal),*),
                $color_2:ident $color_2_repr:literal ($($piece_2_repr:literal),*);
    ) => {
        $crate::interface::game_details!(
            @bitboard
            u64 {
                Square = Square;

                FirstFile = $crate::interface::derive_set!(@first_file BitBoard);
                FirstRank = $crate::interface::derive_set!(@first_rank BitBoard);
            }
        );

        $crate::interface::game_details!(
            @bitboard_less
            Files: $($file_variant),* ;
            Ranks: $($rank_number $rank_variant),* ;

            Pieces: $($piece_variant $piece_repr),*;
                    $($other_variant $other_repr),*;
            Colors: $color_1 $color_1_repr ($($piece_1_repr),*),
                    $color_2 $color_2_repr ($($piece_2_repr),*);
        );
    };

    // @bitboard_less generates all the board representation backing types
    // except BitBoard from the given game specific information.
    (
        @bitboard_less
        Files: $($file_variant:ident),* ;
        Ranks: $($rank_number:literal $rank_variant:ident),* ;

        Pieces: $($piece_variant:ident $piece_repr:literal),*;
                $($other_variant:ident $other_repr:literal),*;
        Colors: $color_1:ident $color_1_repr:literal ($($piece_1_repr:literal),*),
                $color_2:ident $color_2_repr:literal ($($piece_2_repr:literal),*);
    ) => {
        // Square types.
        $crate::interface::game_details!(
            @squares
            Files: $($file_variant),* ;
            Ranks: $($rank_number $rank_variant),* ;
        );

        // Piece types.
        $crate::interface::game_details!(
            @pieces
            Pieces: $($piece_variant $piece_repr),*;
                    $($other_variant $other_repr),*;
            Colors: $color_1 $color_1_repr ($($piece_1_repr),*),
                    $color_2 $color_2_repr ($($piece_2_repr),*);
        );
    };

    // @bitboard generates the BitBoard type from the given game details.
    (@bitboard $typ:tt {
        Square = $sq:tt;
        FirstFile = $first_file:expr;
        FirstRank = $first_rank:expr;
    }) => {
        // The BitBoard type. BitBoardType requires conformance to the SetType
        // trait which is why the declaration uses the set_type macro.
        crate::interface::set_type!(BitBoard<$sq>: $typ);

        // Conformance to BitBoardType.
        impl crate::interface::BitBoardType for BitBoard {
            type Base = $typ;
            type Square = $sq;

            const FIRST_FILE: Self = $first_file;
            const FIRST_RANK: Self = $first_rank;
        }

        // Display a bitboard as ASCII art with 0s and 1s.
        impl std::fmt::Display for BitBoard {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use $crate::interface::{SquareType, SetType};
                use strum::IntoEnumIterator;

                // Iterate over the Ranks in reversed order since we are
                // printing top to bottom and the top rank is the last Rank.
                for rank in <$sq as SquareType>::Rank::iter().rev() {
                    for file in <$sq as SquareType>::File::iter() {
                        let square = $sq::new(file, rank);
                        write!(f, "{}", if self.contains(square) {
                            "1 " // 1 if the BitBoard contains the Square
                        } else {
                            "0 " // 0 if the BitBoard doesn't contain the Square
                        })?;
                    }

                    writeln!(f)?;
                }

                Ok(())
            }
        }

        impl std::fmt::Debug for BitBoard {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self)
            }
        }
    };

    // @pieces generates the piece types, which include Piece, Color, and
    // ColoredPiece from the given game specific details like their string
    // representations.
    (
        @pieces
        Pieces: $($piece_variant:ident $piece_repr:literal),*;
                $($other_variant:ident $other_repr:literal),*;
        Colors: $color_1:ident $color_1_repr:literal ($($piece_1_repr:literal),*),
                $color_2:ident $color_2_repr:literal ($($piece_2_repr:literal),*);
    ) => {
        $crate::interface::game_details!(
            @color $color_1 $color_1_repr; $color_2 $color_2_repr;
        );

        // The Piece type.
        $crate::interface::representable_type!(
            enum Piece: u8 {
                $($piece_variant $piece_repr,)*
                $($other_variant $other_repr,)*
            }
        );

        // The ColoredPiece type.
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
                        // For colored pieces, the ColoredPiece with the same
                        // ::piece() but different ::color() should map to the
                        // same Piece variant.
                        $(
                            Self:: [< $color_1 $piece_variant >] | // Color 1 Piece
                            Self:: [< $color_2 $piece_variant >]   // Color 2 Piece
                                => Self::Piece::$piece_variant,
                        )*

                        // For uncolored pieces, each variant maps to a unique
                        // Piece variant.
                        $(Self:: $other_variant => Self::Piece::$other_variant)*
                    }
                )
            }

            #[allow(unreachable_patterns)]
            fn color(self) -> Self::Color {
                paste::paste!(
                    match self {
                        // For colored pieces, all variants with a common color
                        // map to that single Color variant.
                        $(Self:: [< $color_1 $piece_variant >])|* => Self::Color::$color_1,
                        $(Self:: [< $color_2 $piece_variant >])|* => Self::Color::$color_2,

                        // Uncolored pieces don't have a Color, so panic if
                        // ::color() is called on one. TODO: maybe return Option?
                        _ => panic!("ColoredPiece::color() called on uncolored piece")
                    }
                )
            }
        }
    };

    // @color generates a Color type conforming to the ColorType trait.
    (
        @color
        $first:tt $first_repr:expr;
        $second:tt $second_repr:expr;
    ) => {
        // ColorType requires conformance to RepresentableType<u8>.
        crate::interface::representable_type! {
            enum Color: u8 {
                $first $first_repr, $second $second_repr,
            }
        }

        // ColorType requires conformance to ops::Not.
        impl std::ops::Not for Color {
            type Output = Self;

            fn not(self) -> Self::Output {
                use $crate::interface::RepresentableType;
                unsafe { Self::unsafe_from(self as usize ^ 1) }
            }
        }

        // Other methods needed for ColorType conformance.
        impl crate::interface::ColorType for Color {
            const FIRST: Self = Self::$first;
        }
    };

    // @squares generates the square types, which include Square, File, and Rank
    // from the given game specific details like the number of Files and Ranks.
    (
        @squares
        Files: $($file_variant:ident),* ;
        Ranks: $($rank_number:literal $rank_variant:ident),* ;
    ) => {
        // The Square type's variants are the cartesian product of the variants
        // of its File and Rank types.
        $crate::interface::game_details!(
            @file_rank_product $($rank_number),*;$($file_variant),*
        );

        // SquareType implementation for Square.
        impl $crate::interface::SquareType for Square {
            type File = File;
            type Rank = Rank;
        }

        // The File type. A custom display method is implemented so the
        // @no_display decorator for representable_type! is used.
        $crate::interface::representable_type!(
            @no_display enum File: u8 {
                $($file_variant,)*
            }
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
                        use $crate::interface::RepresentableType;
                        // Files are represented by small letters from the
                        // English alphabet, starting from 'a'.
                        let file_idx = s.chars().next().unwrap_unchecked() as u8 - 'a' as u8;
                        if file_idx < File::N as u8 {
                                Ok(File::unsafe_from(file_idx))
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

        // The Rank type. A custom display method is implemented so the
        // @no_display decorator for representable_type! is used.
        $crate::interface::representable_type!(
            @no_display enum Rank: u8 {
                $($rank_variant,)*
            }
        );

        impl std::str::FromStr for Rank {
            type Err = $crate::interface::TypeParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if let Ok(rank_idx) = s.parse::<u8>() {
                    if rank_idx < Rank::N as u8 {
                        Ok(unsafe {
                            // The first rank is represented as 0 so subtract 1.
                            Rank::unsafe_from(rank_idx - 1)
                        })
                    } else {
                        Err($crate::interface::TypeParseError::StrError(
                            stringify!(Rank).to_string()
                        ))
                    }
                } else {
                    Err($crate::interface::TypeParseError::StrError(
                        stringify!(Rank).to_string()
                    ))
                }
            }
        }

        impl std::fmt::Display for Rank {
            #[allow(clippy::char_lit_as_u8)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", *self as u8 + 1)
            }
        }
    };

    // @file_rank_product generates the Square type, populating its variants
    // with the cartesian product of the Rank and File variants.
    (@file_rank_product $($e1:expr),* ; $($e2:expr),*) => {
        // The Square type. The @cartesian decorator is used to instruct
        // representable_type! to populate the variants with the cartesian
        // product of the two provided sets of variants.
        $crate::interface::representable_type!(
            @cartesian Square, u8; [$([$e1])*][$([$e2])*]
        );

        impl std::str::FromStr for Square {
            type Err = $crate::interface::TypeParseError;

            #[allow(clippy::char_lit_as_u8)]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.len() < 2 {
                    Err($crate::interface::TypeParseError::StrError(
                        stringify!(File).to_string()
                    ))
                } else {
                    // A Square is represented by a Rank and a File.
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

// The representable_type macro generates a type conforming to the
// RepresentableType trait.
macro_rules! representable_type {
    (enum $type:tt: $base:tt {
        $($variant:tt $repr:expr,)*
    }) => {
        $crate::interface::representable_type! {
            @no_display enum $type: $base {
                $($variant,)*
            }
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

    // @no_display suppresses the generation of the FromStr and Display traits
    // for the given RepresentableType. When @no_display is used, a manual
    // implementation of the two traits need to be provided.
    (@no_display enum $type:tt: $base:tt {
        $($variant:tt,)*
    }) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug, strum_macros::EnumIter)]
        #[repr($base)]
        pub enum $type { $($variant,)* }

        impl $crate::interface::RepresentableType<$base> for $type {
            const N: usize = 0 $(+ $crate::interface::representable_type!(@__puke_1 $variant))*;
        }

        $crate::interface::representable_type!(@__impl $type $base);
    };

    // @cartesian populates the variants of the given RepresentableType with the
    // cartesian product of the two provided sets of variants.
    // Input format: [[1] [2] [3]] [[A] [B] [C]]
    (@cartesian $type:ident, $base:tt; [$([$e1:expr])*]$e2:tt) => {
        $crate::interface::representable_type!(
            @__cartesian_helper $type, $base; $([[$e1]$e2])*
        );
        $crate::interface::representable_type!(@__impl $type $base);
    };

    // Input format: [[1] [[A] [B] [C]]] [[2] [[A] [B] [C]]]
    (@__cartesian_helper $type:ident, $base:tt; $([[$e1:expr][$([$e2:expr])*]])*) => {
        paste::paste! {
            #[derive(Copy, Clone, PartialEq, Eq, Debug, strum_macros::EnumIter)]
            #[repr($base)]
            pub enum $type {
                $($([<$e2 $e1>]),*),*
            }
        }

        impl $crate::interface::RepresentableType<$base> for $type {
            const N: usize = 0 $($(+ $crate::interface::representable_type!(@__puke_1 $e1 $e2))*)*;
        }
    };

    (@__impl $type:ident $base:tt) => {
        impl From<$type> for $base {
            #[must_use]
            fn from(value: $type) -> Self {
                value as $base
            }
        }

        impl TryFrom<$base> for $type {
            type Error = $crate::interface::TypeParseError;

            fn try_from(value: $base) -> Result<Self, Self::Error> {
                use $crate::interface::RepresentableType;
                if value as usize >= Self::N {
                    Err(
                        $crate::interface::TypeParseError::RangeError(
                            stringify!($type).to_string()
                        )
                    )
                } else {
                    Ok(unsafe { Self::unsafe_from(value) })
                }
            }
        }

        impl<A, const N: usize> std::ops::Index<$type> for [A; N] {
            type Output = A;

            fn index(&self, index: $type) -> &Self::Output {
                &self[u8::from(index) as usize]
            }
        }

        impl<A, const N: usize> std::ops::IndexMut<$type> for [A; N] {
            fn index_mut(&mut self, index: $type) -> &mut Self::Output {
                &mut self[u8::from(index) as usize]
            }
        }
    };

    (@__puke_1 $($t:tt)*) => { 1 };
}

pub(crate) use representable_type;

/// bitboard_type generates a new BitBoard with the given type as its base
/// representation. The base type must implement num_traits::int::PrimInt so
/// that the BitBoardType trait can be implemented.
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
            derive_more::From,
            derive_more::Into,
        )]
        pub struct $name(pub $typ);

        impl crate::interface::SetType<$typ, $sq> for $name {
            const EMPTY: Self = Self(0);
            const UNIVERSE: Self = $crate::interface::derive_set!(@universe $typ $sq);
        }

        impl Iterator for $name {
            type Item = $sq;

            /// next pops the next Square from the BitBoard and returns it.
            fn next(&mut self) -> Option<Self::Item> {
                use $crate::interface::{RepresentableType, SetType};
                let lsb = if $name::is_empty(*self) {
                    None
                } else {
                    let sq = <Self as Into<$typ>>::into(
                        *self,
                    )
                    .trailing_zeros() as usize;
                    Some(unsafe {
                        $sq::unsafe_from(sq)
                    })
                };

                if !$name::is_empty(*self) {
                    let copy = $typ::from(*self);
                    self.0 = copy & (copy - $typ::from(1u8));
                }

                lsb
            }
        }

        // a -> {a}
        impl From<$sq> for $name {
            #[must_use]
            fn from(square: $sq) -> Self {
                Self($typ::from(1u8) << u8::from(square))
            }
        }

        // The set complement operator (!).
        impl std::ops::Not for $name {
            type Output = Self;

            /// Returns the complementary BitBoard of `self`.
            #[must_use]
            fn not(self) -> Self::Output {
                use $crate::interface::SetType;
                // ! will set the unused bits so remove them with an &.
                Self(!self.0) & Self::UNIVERSE
            }
        }

        // The set difference operator (-).
        #[allow(clippy::suspicious_arithmetic_impl)]
        impl std::ops::Sub for $name {
            type Output = Self;

            /// Returns the difference of `self` and `rhs` as a new BitBoard.
            #[must_use]
            fn sub(self, rhs: Self) -> Self::Output {
                self & !rhs
            }
        }

        // Assignment version of the set difference operator.
        #[allow(clippy::suspicious_arithmetic_impl)]
        impl std::ops::SubAssign for $name {
            /// Returns the difference of `self` and `rhs` as a new BitBoard.
            fn sub_assign(&mut self, rhs: Self) {
                *self &= !rhs
            }
        }

        // A | {a}
        #[allow(clippy::suspicious_arithmetic_impl)]
        impl std::ops::BitOr<$sq> for $name {
            type Output = Self;

            /// Returns the union of `self` and `rhs` as a new BitBoard.
            #[must_use]
            fn bitor(self, rhs: $sq) -> Self::Output {
                self | Self::from(rhs)
            }
        }

        // A - {a}
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

// derive_set resolves into some of the fundamental sets for a SetType/BitBoard.
macro_rules! derive_set {
    // @universe resolves into the universal set.
    (@universe $base:tt $elem:tt) => {{
        use $crate::interface::RepresentableType;
        Self(
            // (1 << Square::N) - 1
            match (1 as $base)
                .checked_shl(<$elem as RepresentableType<u8>>::N as u32)
            {
                Some(universe) => universe.wrapping_sub(1),
                None => (-1i8) as $base,
            },
        )
    }};

    // @first_rank resolves into the set containing the Squares in the first Rank.
    (@first_rank $bb:tt) => {{
        use $crate::interface::{RepresentableType, BitBoardType, SquareType};
        Self(
            // (1 << File::N) - 1
            match (1 as <$bb as BitBoardType>::Base)
                .checked_shl(<<<$bb as BitBoardType>::Square as SquareType>::File as RepresentableType<u8>>::N as u32) {
                    Some(universe) => universe.wrapping_sub(1),
                    None => 1,
                }
        )
    }};

    // @first_file resolves into the set containing the Squares in the first File.
    (@first_file $bb:tt) => {{
        use $crate::interface::{RepresentableType, BitBoardType, SquareType};

        let mut i = 0u32;
        let mut file = 0 as <$bb as BitBoardType>::Base;
        let file_n = <<
            <$bb as BitBoardType>::Square as SquareType
        >::File as RepresentableType<u8>>::N as u32;

        loop {
            // i < file_n
            if i >= file_n {
                break
            }

            // file |= 1 << (File::N * i)
            file |= match (1 as <$bb as BitBoardType>::Base).checked_shl(file_n * i) {
                Some(file) => file,
                None => 0,
            };

            // i++
            i += 1;
        }
        Self(file)
    }};
}
pub(crate) use derive_set;
