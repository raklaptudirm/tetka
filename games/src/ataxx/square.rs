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
use std::str::FromStr;

use crate::interface::{representable_type, RepresentableType, SquareType};

representable_type!(
    /// Square represents all the squares present on an Ataxx Board.
    /// The index of each Square is equal to `rank-index * 8 + file-index`.
    super enum Square: u8 {
        A1 B1 C1 D1 E1 F1 G1
        A2 B2 C2 D2 E2 F2 G2
        A3 B3 C3 D3 E3 F3 G3
        A4 B4 C4 D4 E4 F4 G4
        A5 B5 C5 D5 E5 F5 G5
        A6 B6 C6 D6 E6 F6 G6
        A7 B7 C7 D7 E7 F7 G7
    }
);

impl SquareType for Square {
    type File = File;
    type Rank = Rank;
}

representable_type!(
    /// File represents a file on the Ataxx Board. Each vertical column of Squares
    /// on an Ataxx Board is known as a File. There are 7 of them in total.
    super enum File: u8 { A B C D E F G }
);

representable_type!(
    /// Rank represents a rank on the Ataxx Board. Each horizontal row of Squares
    /// on an Ataxx Board is known as a Rank. There are 7 of them in total.
    enum Rank: u8 {
        First "1", Second "2", Third "3", Fourth "4", Fifth "5", Sixth "6", Seventh "7",
    }
);
