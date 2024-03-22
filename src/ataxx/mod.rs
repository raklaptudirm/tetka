// Non-namespaced modules.
mod bitboard;
mod color;
mod square;
mod board;
mod fen;
mod r#move;

// Namespaced modules.
pub mod hash;

// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bitboard::*;
pub use self::color::*;
pub use self::square::*;
pub use self::board::*;
pub use self::r#move::*;
pub use self::fen::*;
