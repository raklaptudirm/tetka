// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bitboard::*;
pub use self::piece::*;
pub use self::position::*;
pub use self::r#move::*;
pub use self::square::*;

// Non-namespaced modules.
mod bitboard;
mod r#move;
mod piece;
mod position;
mod square;

pub mod moves;
