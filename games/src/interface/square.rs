use super::RepresentableType;

pub trait SquareType: RepresentableType<u8>
where
    Self::File: RepresentableType<u8>,
    Self::Rank: RepresentableType<u8>,
{
    type File;
    type Rank;

    fn new(file: Self::File, rank: Self::Rank) -> Self {
        unsafe { Self::unsafe_from(rank.into() * Self::File::N as u8 + file.into()) }
    }

    fn file(self) -> Self::File {
        unsafe { Self::File::unsafe_from(self.into() % Self::File::N as u8) }
    }

    fn rank(self) -> Self::Rank {
        unsafe { Self::Rank::unsafe_from(self.into() / Self::File::N as u8) }
    }

    fn north(self) -> Option<Self> {
        if self.rank().into() as usize == Self::Rank::N - 1 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() + Self::File::N as u8) })
        }
    }

    fn south(self) -> Option<Self> {
        if self.rank().into() as usize == 0 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() - Self::File::N as u8) })
        }
    }

    fn east(self) -> Option<Self> {
        if self.file().into() as usize == Self::File::N - 1 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() + 1) })
        }
    }

    fn west(self) -> Option<Self> {
        if self.file().into() as usize == 0 {
            None
        } else {
            Some(unsafe { Self::unsafe_from(self.into() - 1) })
        }
    }
}
