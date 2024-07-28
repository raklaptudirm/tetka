// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bitboard::*;
pub use self::hash::*;
pub use self::perft::*;
pub use self::piece::*;
pub use self::position::*;
pub use self::r#move::*;
pub use self::square::*;

// Non-namespaced modules.
mod bitboard;
mod hash;
mod r#move;
mod perft;
mod piece;
mod position;
mod square;

pub mod moves;
