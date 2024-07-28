use std::fmt::Display;
use std::str::FromStr;

mod bitboard;
mod r#move;
mod piece;
mod position;
mod square;

pub use bitboard::*;
pub use piece::*;
pub use position::*;
pub use r#move::*;
pub use square::*;

/// RepresentableType is a basic trait which is implemented by enums with both a
/// binary and string representation and backed by an integer.
pub trait RepresentableType<B: Into<usize>>:
    Copy + Eq + FromStr + Display + From<B> + Into<B>
{
    /// N is the number of specializations of the enum.
    const N: usize;

    /// unsafe_from unsafely converts the given number into Self.
    fn unsafe_from<T: Copy + Into<usize>>(number: T) -> Self {
        debug_assert!(number.into() < Self::N);
        unsafe { std::mem::transmute_copy(&number) }
    }
}
