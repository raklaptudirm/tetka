// Copyright © 2024 Rak Laptudirm <rak@laptudirm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::ops;

use crate::interface::{game_details, RepresentableType, SquareType};

game_details!(
    @squares
    Files: A, B, C, D, E, F, G, H;
    Ranks: 1 First, 2 Second, 3 Third, 4 Fourth, 5 Fifth, 6 Sixth, 7 Seventh, 8 Eighth;
);

use super::Color;

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
