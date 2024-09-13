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

use std::fmt;

use crate::interface::{representable_type, RepresentableType, SquareType};

use super::Color;

representable_type!(
    /// Square represents all the squares present on an Ataxx Board.
    /// The index of each Square is equal to `rank-index * 8 + file-index`.
    enum Square: u8 {
        A1 "a1", B1 "b1", C1 "c1", D1 "d1", E1 "e1", F1 "f1", G1 "g1", H1 "h1",
        A2 "a2", B2 "b2", C2 "c2", D2 "d2", E2 "e2", F2 "f2", G2 "g2", H2 "h2",
        A3 "a3", B3 "b3", C3 "c3", D3 "d3", E3 "e3", F3 "f3", G3 "g3", H3 "h3",
        A4 "a4", B4 "b4", C4 "c4", D4 "d4", E4 "e4", F4 "f4", G4 "g4", H4 "h4",
        A5 "a5", B5 "b5", C5 "c5", D5 "d5", E5 "e5", F5 "f5", G5 "g5", H5 "h5",
        A6 "a6", B6 "b6", C6 "c6", D6 "d6", E6 "e6", F6 "f6", G6 "g6", H6 "h6",
        A7 "a7", B7 "b7", C7 "c7", D7 "d7", E7 "e7", F7 "f7", G7 "g7", H7 "h7",
        A8 "a8", B8 "b8", C8 "c8", D8 "d8", E8 "e8", F8 "f8", G8 "g8", H8 "h8",
    }
);

impl SquareType for Square {
    type File = File;
    type Rank = Rank;
}

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

    pub fn diagonal(self) -> usize {
        14 - self.rank() as usize - self.file() as usize
    }

    pub fn anti_diagonal(self) -> usize {
        7 - self.rank() as usize + self.file() as usize
    }
}

representable_type!(
    /// File represents a file on the Chess Board. Each vertical column of Squares
    /// on an Chess Board is known as a File. There are 7 of them in total.
    enum File: u8 { A "a", B "b", C "c", D "d", E "e", F "f", G "g", H "h", }
);

representable_type!(
    /// Rank represents a rank on the Chess Board. Each horizontal row of Squares
    /// on an Chess Board is known as a Rank. There are 7 of them in total.
    enum Rank: u8 {
        First "1", Second "2", Third "3", Fourth "4",
        Fifth "5", Sixth "6", Seventh "7", Eighth "8",
    }
);
