use super::RepresentableType;

/// SquareType is the trait implemented by the type for Squares in a game's
/// board representation. It provides a generic interface along with useful
/// predefined methods that a Square type would require.
pub trait SquareType: RepresentableType<u8>
where
    Self::File: RepresentableType<u8>,
    Self::Rank: RepresentableType<u8>,
{
    type File;
    type Rank;

    /// new creates a new Square from the given File and Rank.
    fn new(file: Self::File, rank: Self::Rank) -> Self {
        unsafe { Self::unsafe_from(rank.into() * Self::File::N as u8 + file.into()) }
    }

    /// Returns the File of self.
    fn file(self) -> Self::File {
        unsafe { Self::File::unsafe_from(self.into() % Self::File::N as u8) }
    }

    /// Returns the Rank of self.
    fn rank(self) -> Self::Rank {
        unsafe { Self::Rank::unsafe_from(self.into() / Self::File::N as u8) }
    }

    /// Returns the square to the north of self. If there is no Square to the
    /// north of self, it returns None.
    fn north(self) -> Option<Self> {
        if self.rank().into() as usize == Self::Rank::N - 1 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() + Self::File::N as u8) })
        }
    }

    /// Returns the square to the south of self. If there is no Square to the
    /// south of self, it returns None.
    fn south(self) -> Option<Self> {
        if self.rank().into() as usize == 0 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() - Self::File::N as u8) })
        }
    }

    /// Returns the square to the east of self. If there is no Square to the
    /// east of self, it returns None.
    fn east(self) -> Option<Self> {
        if self.file().into() as usize == Self::File::N - 1 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() + 1) })
        }
    }

    /// Returns the square to the west of self. If there is no Square to the
    /// west of self, it returns None.
    fn west(self) -> Option<Self> {
        if self.file().into() as usize == 0 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() - 1) })
        }
    }
}
