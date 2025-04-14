// Copyright © 2023 Rak Laptudirm <rak@laptudirm.com>
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

use std::sync::LazyLock;

use super::{BitBoard, Color, Direction, Square};
use crate::interface::{BitBoardType, RepresentableType, SetType, SquareType};

use strum::IntoEnumIterator;

pub fn pawn_attacks(square: Square, color: Color) -> BitBoard {
    PAWN_ATTACKS_TABLE[color as usize][square as usize]
}

pub fn knight(square: Square) -> BitBoard {
    KNIGHT_MOVES_TABLE[square as usize]
}

pub fn bishop(square: Square, blockers: BitBoard) -> BitBoard {
    hyperbola(square, blockers, BitBoard::diagonal(square.diagonal()))
        | hyperbola(
            square,
            blockers,
            BitBoard::anti_diagonal(square.anti_diagonal()),
        )
}

pub fn rook(square: Square, blockers: BitBoard) -> BitBoard {
    hyperbola(square, blockers, BitBoard::file(square.file()))
        | hyperbola(square, blockers, BitBoard::rank(square.rank()))
}

pub fn queen(square: Square, blockers: BitBoard) -> BitBoard {
    bishop(square, blockers) | rook(square, blockers)
}

pub fn king(square: Square) -> BitBoard {
    KING_MOVES_TABLE[square as usize]
}

pub(crate) fn hyperbola(
    square: Square,
    blockers: BitBoard,
    mask: BitBoard,
) -> BitBoard {
    let mask = mask.0;
    let square = BitBoard::from(square).0;
    let rev_sq = square.reverse_bits();
    let blockers = blockers.0;

    let mut ray = blockers & mask;
    let mut rev = ray.reverse_bits();
    ray = ray.wrapping_sub(square.wrapping_mul(2));
    rev = rev.wrapping_sub(rev_sq.wrapping_mul(2));
    ray ^= rev.reverse_bits();
    ray &= mask;

    BitBoard(ray)
}

static KING_MOVES_TABLE: LazyLock<[BitBoard; Square::N]> =
    LazyLock::new(|| {
        let mut king_moves = [BitBoard::EMPTY; Square::N];
        for square in Square::iter() {
            let square_bb = BitBoard::from(square);
            let line = square_bb | square_bb.east() | square_bb.west();
            let cell = line | line.north() | line.south();
            king_moves[square as usize] = cell ^ square_bb;
        }

        king_moves
    });

static KNIGHT_MOVES_TABLE: LazyLock<[BitBoard; Square::N]> =
    LazyLock::new(|| {
        let mut knight_moves = [BitBoard::EMPTY; Square::N];
        for square in Square::iter() {
            let square_bb = BitBoard::from(square);

            let east = square_bb.east().east();
            let west = square_bb.west().west();

            let east = east.north() | east.south();
            let west = west.north() | west.south();

            let north = square_bb.north().north();
            let south = square_bb.south().south();

            let north = north.east() | north.west();
            let south = south.east() | south.west();

            knight_moves[square as usize] = north | south | east | west;
        }

        knight_moves
    });

static PAWN_ATTACKS_TABLE: LazyLock<[[BitBoard; Square::N]; Color::N]> =
    LazyLock::new(|| {
        let mut pawn_attacks = [[BitBoard::EMPTY; Square::N]; Color::N];

        for color in Color::iter() {
            for square in Square::iter() {
                let square_bb_up =
                    BitBoard::from(square).shift(Direction::up(color));
                pawn_attacks[color as usize][square as usize] =
                    square_bb_up.east() | square_bb_up.west();
            }
        }

        pawn_attacks
    });
