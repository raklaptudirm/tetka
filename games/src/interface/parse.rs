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

use std::num::ParseIntError;
use std::str::FromStr;

use crate::interface::{
    ColorType, ColoredPiece, File, PositionType, Rank, Square, SquareType,
};

use strum::IntoEnumIterator;
use thiserror::Error;

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

pub(crate) fn piece_placement<T: PositionType>(
    position: &mut T,
    fen_fragment: &str,
) -> Result<(), PiecePlacementParseError> {
    for sq in T::Square::iter() {
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
            return Err(PiecePlacementParseError::TooManyRanks(
                Rank::<T>::iter().len(),
            ));
        }

        // Iterate over the Square specs in the Rank spec.
        for data in rank_data.chars() {
            // Check if a jump was too big and we landed on an invalid File.
            if file.is_err() {
                return Err(PiecePlacementParseError::JumpTooLong);
            }

            let file_value = *file.as_ref().unwrap();
            let rank_value = *rank.as_ref().unwrap();
            let square = Square::<T>::new(file_value, rank_value);
            match data {
                // Numbers represent jump specs to jump over empty squares.
                '1'..='8' => {
                    file = File::<T>::try_from(
                        file_value.into() + data as u8 - b'1',
                    );
                    if file.is_err() {
                        return Err(PiecePlacementParseError::JumpTooLong);
                    }
                }

                _ => match ColoredPiece::<T>::from_str(&data.to_string()) {
                    Ok(piece) => position.insert(square, piece),
                    Err(_) => {
                        return Err(
                            PiecePlacementParseError::InvalidPieceIdent(data),
                        )
                    }
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
        rank = Rank::<T>::try_from((rank.unwrap().into()).wrapping_sub(1));
        file = Ok(first_file);
    }

    Ok(())
}

pub(crate) fn ply_count<C: ColorType>(
    fmc: &str,
    stm: C,
) -> Result<u16, ParseIntError> {
    if stm == C::FIRST {
        Ok(fmc.parse::<u16>()? * 2 - 1)
    } else {
        Ok(fmc.parse::<u16>()? * 2 - 2)
    }
}
