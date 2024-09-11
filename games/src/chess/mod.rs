// Namespaced modules.
pub mod castling;
pub mod moves;
pub mod zobrist;

// Non-namespaced modules.
mod bitboard;
mod color;
mod r#move;
mod position;
mod square;

// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bitboard::*;
pub use self::color::*;
pub use self::position::*;
pub use self::r#move::*;
pub use self::square::*;
