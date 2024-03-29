// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bitboard::*;
pub use self::board::*;
pub use self::color::*;
pub use self::fen::*;
pub use self::hash::*;
pub use self::r#move::*;
pub use self::square::*;

// Non-namespaced modules.
mod bitboard;
mod color;
mod square;
mod board;
mod fen;
mod r#move;
mod hash;
