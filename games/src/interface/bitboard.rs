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
        (self << 1)
            & (Self::UNIVERSE
                ^ unsafe {
                    Self::file(<Self::Square as SquareType>::File::unsafe_from(
                        0u8,
                    ))
                })
    }

    /// west returns a new Self with all the squares shifted to the west.
    #[must_use]
    fn west(self) -> Self {
        (self >> 1)
            & (Self::UNIVERSE
                ^ unsafe {
                    Self::file(<Self::Square as SquareType>::File::unsafe_from(
                        <Self::Square as SquareType>::File::N as u8 - 1,
                    ))
                })
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
