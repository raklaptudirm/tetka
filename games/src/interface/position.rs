use std::fmt::Display;
use std::str::FromStr;

use strum::IntoEnumIterator;
use thiserror::Error;

use super::{BitBoardType, ColoredPieceType, Hash, MoveList, MoveStore, MoveType, SquareType};

/// Position is a generalized interface for board representations of a wide
/// range of games. It can be used to create game-agnostic software. Tetka
/// provides some of the popular board representations out of the box, but
/// custom ones can also be implemented by the library user.
pub trait PositionType: FromStr + Display
where
    Self::BitBoard: BitBoardType,
    Self::ColoredPiece: ColoredPieceType,
    Self::Move: MoveType,
{
    /// The type for the bitboards used by this board representation.
    type BitBoard;

    /// The type for the pieces (with color) used by this board representation.
    type ColoredPiece;

    /// The type for one move in this board representation.
    type Move;

    // Peeking, insertion, and removal of pieces from the board representation.

    /// insert puts the given piece at the given square. The implementation is
    /// free to assume that the square is currently empty.
    fn insert(&mut self, sq: <Self::BitBoard as BitBoardType>::Square, piece: Self::ColoredPiece);
    /// remove clears the given square, and returns the piece that was there.
    fn remove(
        &mut self,
        sq: <Self::BitBoard as BitBoardType>::Square,
    ) -> Option<Self::ColoredPiece>;
    /// at returns the piece that is in the given square.
    fn at(&self, sq: <Self::BitBoard as BitBoardType>::Square) -> Option<Self::ColoredPiece>;

    fn piece_bb(&self, piece: <Self::ColoredPiece as ColoredPieceType>::Piece) -> Self::BitBoard;
    fn color_bb(&self, color: <Self::ColoredPiece as ColoredPieceType>::Color) -> Self::BitBoard;
    fn colored_piece_bb(&self, piece: Self::ColoredPiece) -> Self::BitBoard;

    fn hash(&self) -> Hash;

    // Game Result functions.

    /// winner returns the winning side in the current position.
    fn winner(&self) -> Option<<Self::ColoredPiece as ColoredPieceType>::Color>;
    /// is_game_over returns a boolean representing if the game is over.
    fn is_game_over(&self) -> bool {
        self.winner().is_some()
    }

    /// after_move returns the position after playing the given move on the
    /// current position. The UPDATE_PERIPHERALS flag can be interpreted as
    /// toggling the non-essential updated which are done by this function, like
    /// the hash function for the position.
    fn after_move<const UPDATE_PERIPHERALS: bool>(&self, mov: Self::Move) -> Self;

    // Move Generation functions for the board representation.

    /// generate_moves_into generates all the moves in the current position into
    /// the given move storage. The QUIET and NOISY flags toggles the generation
    /// of reversible and irreversible moves respectively.
    fn generate_moves_into<
        const ALLOW_ILLEGAL: bool,
        const QUIET: bool,
        const NOISY: bool,
        T: MoveStore<Self::Move>,
    >(
        &self,
        movelist: &mut T,
    );
    /// generate_moves is similar to generate_moves_into, except that instead of
    /// taking some storage as input it stores into a custom stack-based type.
    fn generate_moves<const ALLOW_ILLEGAL: bool, const QUIET: bool, const NOISY: bool>(
        &self,
    ) -> MoveList<Self::Move> {
        let mut movelist: MoveList<Self::Move> = Default::default();
        self.generate_moves_into::<ALLOW_ILLEGAL, QUIET, NOISY, _>(&mut movelist);
        movelist
    }
    /// count_moves is similar to generate_moves, except instead of returning a
    /// list of the available moves, it returns the number of available moves.
    /// By default this is simply `generate_moves().len()`, but implementations
    /// may take advantage of various optimizations counting as opposed to
    /// storing the moves allows to provide a more efficient version.
    fn count_moves<const QUIET: bool, const NOISY: bool>(&self) -> usize {
        self.generate_moves::<false, QUIET, NOISY>().len()
    }
}

/// PositionParseErr represents an error encountered while parsing
/// the given FEN position field into a valid Position.
#[derive(Error, Debug)]
pub enum PiecePlacementParseError {
    #[error("a jump value was too long and overshot")]
    JumpTooLong,

    #[error("invalid piece identifier '{0}'")]
    InvalidPieceIdent(char),
    #[error("insufficient data to fill the entire {0} file")]
    FileDataIncomplete(String),
    #[error("expected {0} ranks, found more")]
    TooManyRanks(usize),
}

type Square<P> = <<P as PositionType>::BitBoard as BitBoardType>::Square;
type File<P> = <Square<P> as SquareType>::File;
type Rank<P> = <Square<P> as SquareType>::Rank;
type ColoredPiece<P> = <P as PositionType>::ColoredPiece;

pub fn parse_piece_placement<T: PositionType>(
    position: &mut T,
    fen_fragment: &str,
) -> Result<(), PiecePlacementParseError> {
    for sq in <<<T as PositionType>::BitBoard as BitBoardType>::Square as IntoEnumIterator>::iter()
    {
        position.remove(sq);
    }

    // Spilt the position spec by the Ranks which are separated by '/'.
    let ranks: Vec<&str> = fen_fragment.split('/').collect();

    let first_file = File::<T>::iter().next().unwrap();

    let mut file = Ok(first_file);
    let mut rank = Ok(Rank::<T>::iter().last().unwrap());

    // Iterate over the Ranks in the string spec.
    for rank_data in ranks {
        // Rank pointer ran out, but data carried on.
        if rank.is_err() {
            return Err(PiecePlacementParseError::TooManyRanks(<<<T as PositionType>::BitBoard as BitBoardType>::Square as SquareType>::Rank::iter().len()));
        }

        // Iterate over the Square specs in the Rank spec.
        for data in rank_data.chars() {
            // Check if a jump spec was too big and we landed on an invalid File.
            if file.is_err() {
                return Err(PiecePlacementParseError::JumpTooLong);
            }

            let file_value = *file.as_ref().unwrap();
            let rank_value = *rank.as_ref().unwrap();
            let square = <Square<T>>::new(file_value, rank_value);
            match data {
                // Numbers represent jump specs to jump over empty squares.
                '1'..='8' => {
                    file = <File<T>>::try_from(file_value.into() + data as u8 - b'1');
                    if file.is_err() {
                        return Err(PiecePlacementParseError::JumpTooLong);
                    }
                }

                _ => match <ColoredPiece<T>>::from_str(&data.to_string()) {
                    Ok(piece) => position.insert(square, piece),
                    Err(_) => return Err(PiecePlacementParseError::InvalidPieceIdent(data)),
                },
            }

            // On to the next Square spec in the Rank spec.
            file = <File<T>>::try_from(file.unwrap().into() + 1);
        }

        // After rank data runs out, file pointer should be
        // at the last file, i.e, rank is completely filled.
        if let Ok(file) = file {
            return Err(PiecePlacementParseError::FileDataIncomplete(
                file.to_string(),
            ));
        }

        // Switch rank pointer and reset file pointer.
        rank = <Rank<T>>::try_from((rank.unwrap().into()).wrapping_sub(1));
        file = Ok(first_file);
    }

    Ok(())
}
