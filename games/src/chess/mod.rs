// Namespaced modules.
pub mod castling;
pub mod moves;
pub mod zobrist;

mod movegen;

// Non-namespaced modules.
mod r#move;
mod position;

// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::position::*;
pub use self::r#move::*;

#[cfg(test)]
mod tests;

use std::ops;
use std::sync::LazyLock;

use crate::interface::SetType;
use crate::interface::{BitBoardType, RepresentableType, SquareType};

use strum::IntoEnumIterator;

crate::interface::game_details!(
    Files: A, B, C, D, E, F, G, H;
    Ranks: 1 First, 2 Second, 3 Third, 4 Fourth, 5 Fifth, 6 Sixth, 7 Seventh, 8 Eighth;
    Pieces: Pawn "p", Knight "n", Bishop "b", Rook "r", Queen "q", King "k";;
    Colors: White "w" ("P", "N", "B", "R", "Q", "K"),
            Black "b" ("p", "n", "b", "r", "q", "k");
);

impl BitBoard {
    pub fn new(raw: u64) -> BitBoard {
        BitBoard(raw)
    }

    pub fn shift(&self, dir: Direction) -> BitBoard {
        match dir {
            Direction::North => self.north(),
            Direction::South => self.south(),
            Direction::NorthNorth => self.north().north(),
            Direction::SouthSouth => self.south().south(),
            Direction::East => self.east(),
            Direction::West => self.west(),
            Direction::NorthEast => self.north().east(),
            Direction::NorthWest => self.north().west(),
            Direction::SouthEast => self.south().east(),
            Direction::SouthWest => self.south().west(),
        }
    }
}

impl BitBoard {
    pub fn diagonal(diagonal: usize) -> BitBoard {
        DIAGONAL[diagonal]
    }

    pub fn anti_diagonal(anti_diagonal: usize) -> BitBoard {
        ANTI_DIAGONAL[anti_diagonal]
    }

    pub fn between(sq_1: Square, sq_2: Square) -> BitBoard {
        BETWEEN[sq_1 as usize][sq_2 as usize]
    }

    pub fn between2(sq_1: Square, sq_2: Square) -> BitBoard {
        BETWEEN[sq_1 as usize][sq_2 as usize] | sq_2
    }
}

static DIAGONAL: LazyLock<[BitBoard; 15]> = LazyLock::new(|| {
    const MAIN_DIAGONAL: BitBoard = BitBoard(0x8040201008040201u64);

    let mut diagonals = [BitBoard::EMPTY; 15];

    // Initialize diagonals to the east of the main.
    let mut upper_triangle = MAIN_DIAGONAL;
    for diagonal in diagonals.iter_mut().rev().skip(7) {
        *diagonal = upper_triangle;
        upper_triangle = upper_triangle.east();
    }

    // Initialize diagonals to the west of the main.
    let mut upper_triangle = MAIN_DIAGONAL;
    for diagonal in diagonals.iter_mut().skip(7) {
        *diagonal = upper_triangle;
        upper_triangle = upper_triangle.west();
    }

    diagonals
});

static ANTI_DIAGONAL: LazyLock<[BitBoard; 15]> = LazyLock::new(|| {
    const MAIN_ANTI_DIAGONAL: BitBoard = BitBoard(0x0102040810204080u64);

    let mut anti_diagonals = [BitBoard::EMPTY; 15];

    // Initialize anti-diagonals to the east of the main.
    let mut upper_triangle = MAIN_ANTI_DIAGONAL;
    for anti_diagonal in anti_diagonals.iter_mut().skip(7) {
        *anti_diagonal = upper_triangle;
        upper_triangle = upper_triangle.east();
    }

    // Initialize anti-diagonals to the west of the main.
    let mut upper_triangle = MAIN_ANTI_DIAGONAL;
    for anti_diagonal in anti_diagonals.iter_mut().rev().skip(7) {
        *anti_diagonal = upper_triangle;
        upper_triangle = upper_triangle.west();
    }

    anti_diagonals
});

static BETWEEN: LazyLock<[[BitBoard; Square::N]; Square::N]> =
    LazyLock::new(|| {
        let mut between = [[BitBoard::EMPTY; Square::N]; Square::N];

        for square_1 in Square::iter() {
            for square_2 in Square::iter() {
                let mask = if square_1 == square_2 {
                    // If the two squares are equal their between BitBoard
                    // should be empty since both are excluded from it.
                    continue;
                } else if square_1.file() == square_2.file() {
                    BitBoard::file(square_1.file())
                } else if square_1.rank() == square_2.rank() {
                    BitBoard::rank(square_1.rank())
                } else if square_1.diagonal() == square_2.diagonal() {
                    BitBoard::diagonal(square_1.diagonal())
                } else if square_1.anti_diagonal() == square_2.anti_diagonal() {
                    BitBoard::anti_diagonal(square_1.anti_diagonal())
                } else {
                    continue;
                };

                let blockers = BitBoard::from(square_1) | square_2;

                // This step generates the between BitBoard for the current pair of Squares.
                // We use the mask generated in the previous step to apply the Hyperbola
                // algorithm along the rays joining the Squares together, using the two
                // Squares as the blocker set.
                //
                // Blockers: . x . . . x . .
                // Square 1: x x x x x x . .
                // Square 2: . x x x x x x x
                //
                // The intersection between the two blocked rays will be the between
                // BitBoard + Squares 1 and 2. Therefore, to get the between BitBoard, a
                // final intersection operator with the union of Squares 1 and 2 is do
                between[square_1 as usize][square_2 as usize] =
                    moves::hyperbola(square_1, blockers, mask)
                        & moves::hyperbola(square_2, blockers, mask);
            }
        }

        between
    });

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
