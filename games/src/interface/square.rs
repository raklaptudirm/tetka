use super::RepresentableType;

pub trait Square: RepresentableType<u8>
where
    Self::File: RepresentableType<u8>,
    Self::Rank: RepresentableType<u8>,
{
    type File;
    type Rank;

    fn new(file: Self::File, rank: Self::Rank) -> Self {
        Self::unsafe_from(rank.into() * Self::File::N as u8 + file.into())
    }

    fn file(self) -> Self::File {
        Self::File::unsafe_from(self.into() % Self::File::N as u8)
    }

    fn rank(self) -> Self::Rank {
        Self::Rank::unsafe_from(self.into() / Self::File::N as u8)
    }

    fn north(self) -> Self {
        Self::unsafe_from(self.into() + Self::File::N as u8)
    }

    fn south(self) -> Self {
        Self::unsafe_from(self.into() - Self::File::N as u8)
    }

    fn east(self) -> Self {
        Self::unsafe_from(self.into() + 1)
    }

    fn west(self) -> Self {
        Self::unsafe_from(self.into() - 1)
    }
}
