//! This module provides implementations of the APIs defined in `interface` of
//! various popular games.
//!
//! Each module represents the most general version of a single game. For
//! example, the chess module provides a implementation for DFRC chess, which
//! is a superset of standard chess.
//!
//! The common interfaces defined in `interface` are impelemnted for each game
//! in this list, along with other game-specific functionality, such as
//! types for working with castling rights in chess. Refer to the documentation
//! of `interface` for more information about the common API.

pub mod ataxx;
pub mod chess;
pub mod isolation;
