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

pub use bitboard::BitBoardType;
pub use hash::*;
pub use piece::*;
pub use position::*;
pub use r#move::*;
pub use square::*;

pub(crate) use bitboard::bitboard_type;

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
