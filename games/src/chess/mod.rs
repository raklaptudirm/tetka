// Namespaced modules.
pub mod castling;
pub mod moves;
pub mod zobrist;

mod movegen;

// Non-namespaced modules.
mod bitboard;
mod r#move;
mod position;

// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bitboard::*;
pub use self::position::*;
pub use self::r#move::*;

#[cfg(test)]
mod tests;

use std::ops;

use crate::interface::{RepresentableType, SquareType};

crate::interface::game_details!(
    @bitboard_less
    Files: A, B, C, D, E, F, G, H;
    Ranks: 1 First, 2 Second, 3 Third, 4 Fourth, 5 Fifth, 6 Sixth, 7 Seventh, 8 Eighth;
    Pieces: Pawn "p", Knight "n", Bishop "b", Rook "r", Queen "q", King "k";;
    Colors: White "w" ("P", "N", "B", "R", "Q", "K"),
            Black "b" ("p", "n", "b", "r", "q", "k");
);

impl Square {
    pub fn up(self, stm: Color) -> Option<Square> {
        match stm {
            Color::White => self.north(),
            Color::Black => self.south(),
        }
    }

    pub fn down(self, stm: Color) -> Option<Square> {
        match stm {
            Color::White => self.south(),
            Color::Black => self.north(),
        }
    }

    pub fn shift(self, dir: Direction) -> Square {
        unsafe { Square::unsafe_from((self as i8 + dir as i8) as u8) }
    }

    pub fn diagonal(self) -> usize {
        7 + self.rank() as usize - self.file() as usize
    }

    pub fn anti_diagonal(self) -> usize {
        self.rank() as usize + self.file() as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North = 8,
    South = -8,

    NorthNorth = 8 + 8,
    SouthSouth = -8 - 8,

    East = 1,
    West = -1,

    NorthEast = 8 + 1,
    NorthWest = 8 - 1,
    SouthEast = -8 + 1,
    SouthWest = -8 - 1,
}

impl Direction {
    pub fn up(stm: Color) -> Direction {
        match stm {
            Color::White => Direction::North,
            Color::Black => Direction::South,
        }
    }
}

impl ops::Add for Direction {
    type Output = Direction;

    fn add(self, rhs: Self) -> Self::Output {
        unsafe { std::mem::transmute_copy(&(self as i8 + rhs as i8)) }
    }
}

impl ops::Sub for Direction {
    type Output = Direction;

    fn sub(self, rhs: Self) -> Self::Output {
        unsafe { std::mem::transmute_copy(&(self as i8 - rhs as i8)) }
    }
}

impl ops::Neg for Direction {
    type Output = Direction;

    fn neg(self) -> Self::Output {
        unsafe { std::mem::transmute_copy(&(-(self as i8))) }
    }
}
