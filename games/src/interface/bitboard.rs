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

use num_traits::int::PrimInt;

use super::{RepresentableType, SetType, SquareType};

/// BitBoardType is a generalized interface implemented by BitBoards of
/// arbitrary size. This allows programs to handle BitBoards of any size with
/// generic functions using this common interface.
pub trait BitBoardType: SetType<Self::Base, Self::Square>
where
    Self::Base: PrimInt,
    Self::Square: SquareType,
{
    /// The backing [`PrimInt`] type for the BitBoard.
    type Base;
    /// The type of the Squares in the BitBoard.
    type Square;

    /// The BitBoard containing Squares in the first File.
    const FIRST_FILE: Self;
    /// The BitBoard containing Squares in the first Rank.
    const FIRST_RANK: Self;

    /// north returns a new Self with all the squares shifted to the north.
    #[must_use]
    fn north(self) -> Self {
        (self << <Self::Square as SquareType>::File::N) & Self::UNIVERSE
    }

    /// south returns a new Self with all the squares shifted to the south.
    #[must_use]
    fn south(self) -> Self {
        self >> <Self::Square as SquareType>::File::N
    }

    /// east returns a new Self with all the squares shifted to the east.
    #[must_use]
    fn east(self) -> Self {
        (self << 1) & (Self::UNIVERSE ^ Self::FIRST_FILE)
    }

    /// west returns a new Self with all the squares shifted to the west.
    #[must_use]
    fn west(self) -> Self {
        (self >> 1)
            & (Self::UNIVERSE
                ^ (Self::FIRST_FILE
                    << (<Self::Square as SquareType>::File::N - 1)))
    }

    /// Returns a BitBoard containing all the squares from the given `File`.
    #[must_use]
    fn file(file: <Self::Square as SquareType>::File) -> Self {
        Self::FIRST_FILE << file.into() as usize
    }

    /// Returns a BitBoard containing all the squares from the given `Rank`.
    #[must_use]
    fn rank(rank: <Self::Square as SquareType>::Rank) -> Self {
        Self::FIRST_RANK
            << (<Self::Square as SquareType>::File::N * rank.into() as usize)
    }
}
