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

use crate::interface::bitboard_type;

use super::Square;

bitboard_type! {
    /// A set of Squares implemented as a bitset where the `1 << sq.into()` bit
    /// represents whether `sq` is in the BitBoard or not.
    struct BitBoard : u64 {
        // The BitBoard's Square type.
        Square = Square;

        // BitBoards containing the squares of the first file and the first rank.
        FirstFile = Self(0x0000010101010101);
        FirstRank = Self(0x00000000000000ff);
    }
}

use crate::interface::BitBoardType;

impl BitBoard {
    /// singles returns the targets of all singular moves from all the source
    /// squares given in the provided BitBoard.
    pub fn singles(bb: BitBoard) -> BitBoard {
        let bar = bb | bb.east() | bb.west();
        (bar | bar.north() | bar.south()) ^ bb
    }
}
