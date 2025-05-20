//! tetka-games is a general board game library which provides generic
//! interfaces for creating game agnostic but performant software.
//!
//! The three modules provide different functionality of the library:
//! - `interface`: Provides the generic game agnostic interface for working
//!    with the logic and boards of different games in a single codebase.
//! - `common`: Provides game agnostic utilities which build on top of the apis
//!    provided by `interface`. This includes a `perft` implementation.
//! - `games`: Provides board representations for specific games.

pub mod common;
pub mod games;
pub mod interface;

#[cfg(test)]
mod tests;
