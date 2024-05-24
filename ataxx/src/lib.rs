// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bitboard::*;
pub use self::board::*;
pub use self::hash::*;
pub use self::piece::*;
pub use self::r#move::*;
pub use self::square::*;

// Non-namespaced modules.
mod bitboard;
mod board;
mod hash;
mod r#move;
mod piece;
mod square;

// Private modules.
#[doc(hidden)]
pub mod type_macros;
