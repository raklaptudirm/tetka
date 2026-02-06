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

use std::{fmt::Display, str::FromStr};

use crate::interface::{ColorType, MoveType};

use super::{Hash, MoveList, MoveStore};

/// A generalized interface for board representations of a wide range of games.
///
/// It is designed to create game-agnostic software. Tetka provides the logic
/// for many popular games out of the box, but custom games can easily be
/// implemented by the library user.
pub trait PositionType: FromStr + Display + Default
where
    Self::Move: MoveType<Position = Self>,
    Self::Color: ColorType,
{
    /// Type for one move in this board representation.
    type Move;

    type Color;

    /// FEN string for the standard starting position of the game.
    ///
    /// If the game doesn't have a standard starting a position, any legal
    /// starting position or a de-facto standard may be used.
    const STARTPOS: &str;

    #[must_use]
    fn ply_count(&self) -> usize;
    /// Returns a semi-unique checksum of the current Position.
    #[must_use]
    fn hash(&self) -> Hash;

    /// Returns the side which has won in the current position, if any.
    #[must_use]
    fn winner(&self) -> Option<Option<Self::Color>>;
    /// Returns `true` if the game is over in the current position.
    #[must_use]
    fn is_game_over(&self) -> bool {
        self.winner().is_some()
    }

    /// Returns the position which is reached after playing the given move on
    /// the current position.
    ///
    /// The `UPDATE_PERIPHERALS` flag can be interpreted as toggling the
    /// non-essential updated which are done by this function, like the hash
    /// function for the position.
    #[must_use]
    fn after_move<const UPDATE_PERIPHERALS: bool>(
        &self,
        mov: Self::Move,
    ) -> Self;

    /// Generates all the moves in the current position and add them into the
    ///  given move storage.
    ///
    /// The `ALLOW_ILLEGAL` flag toggles between legal and pseudo-legal move
    /// generation for `false` and `true` respectively.
    ///
    /// The `QUIET` and `NOISY` flags toggles the generation of reversible and
    /// irreversible moves respectively.
    fn generate_moves_into<
        const ALLOW_ILLEGAL: bool,
        const QUIET: bool,
        const NOISY: bool,
        T: MoveStore<Self::Move>,
    >(
        &self,
        movelist: &mut T,
    );
    /// `generate_moves` is similar to `generate_moves_into`, except that
    /// instead of taking some storage as input it stores into a [MoveList].
    #[must_use]
    fn generate_moves<
        const ALLOW_ILLEGAL: bool,
        const QUIET: bool,
        const NOISY: bool,
    >(
        &self,
    ) -> impl MoveStore<Self::Move> {
        let mut movelist: MoveList<Self::Move> = Default::default();
        self.generate_moves_into::<ALLOW_ILLEGAL, QUIET, NOISY, _>(
            &mut movelist,
        );
        movelist
    }
    /// `count_moves` is similar to `generate_moves`, except instead of
    /// returning a list of the available moves, it returns the number of
    /// available moves.
    ///
    /// By default this is simply `generate_moves().len()`, but implementations
    /// may take advantage of various optimizations counting as opposed to
    /// storing the moves allows to provide a more efficient version.
    #[must_use]
    fn count_moves<const QUIET: bool, const NOISY: bool>(&self) -> usize {
        self.generate_moves::<false, QUIET, NOISY>().len()
    }
}
