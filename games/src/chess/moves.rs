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

use crate::chess::{BitBoard, Color, Square};

use crate::interface::RepresentableType;

pub fn pawn_attacks(square: Square, color: Color) -> BitBoard {
    BitBoard::new(PAWN_ATTACKS_TABLE[color as usize][square as usize])
}

pub fn knight(square: Square) -> BitBoard {
    BitBoard::new(KNIGHT_MOVES_TABLE[square as usize])
}

// #[rustfmt::skip]
// #[inline(always)]
// pub fn bishop(square: Square, blockers: BitBoard) -> BitBoard {
//     hyperbola(square, blockers, BitBoard::diagonal(square.diagonal())) |
// 	hyperbola(square, blockers, BitBoard::anti_diagonal(square.anti_diagonal()))
// }

// #[rustfmt::skip]
// #[inline(always)]
// pub fn rook(square: Square, blockers: BitBoard) -> BitBoard {
//     hyperbola(square, blockers, BitBoard::file(square.file())) |
// 	hyperbola(square, blockers, BitBoard::rank(square.rank()))
// }

// #[inline(always)]
// pub fn queen(square: Square, blockers: BitBoard) -> BitBoard {
//     bishop(square, blockers) | rook(square, blockers)
// }

pub fn king(square: Square) -> BitBoard {
    BitBoard::new(KING_MOVES_TABLE[square as usize])
}

#[allow(dead_code)]
fn hyperbola(square: Square, blockers: BitBoard, mask: BitBoard) -> BitBoard {
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

#[rustfmt::skip]
const KING_MOVES_TABLE: [u64; Square::N] = [
    0x0000000000000302, 0x0000000000000705, 0x0000000000000e0a, 0x0000000000001c14,
	0x0000000000003828, 0x0000000000007050, 0x000000000000e0a0, 0x000000000000c040,
	0x0000000000030203, 0x0000000000070507, 0x00000000000e0a0e, 0x00000000001c141c,
	0x0000000000382838, 0x0000000000705070, 0x0000000000e0a0e0, 0x0000000000c040c0,
	0x0000000003020300, 0x0000000007050700, 0x000000000e0a0e00, 0x000000001c141c00,
	0x0000000038283800, 0x0000000070507000, 0x00000000e0a0e000, 0x00000000c040c000,
	0x0000000302030000, 0x0000000705070000, 0x0000000e0a0e0000, 0x0000001c141c0000,
	0x0000003828380000, 0x0000007050700000, 0x000000e0a0e00000, 0x000000c040c00000,
	0x0000030203000000, 0x0000070507000000, 0x00000e0a0e000000, 0x00001c141c000000,
	0x0000382838000000, 0x0000705070000000, 0x0000e0a0e0000000, 0x0000c040c0000000,
	0x0003020300000000, 0x0007050700000000, 0x000e0a0e00000000, 0x001c141c00000000,
	0x0038283800000000, 0x0070507000000000, 0x00e0a0e000000000, 0x00c040c000000000,
	0x0302030000000000, 0x0705070000000000, 0x0e0a0e0000000000, 0x1c141c0000000000,
	0x3828380000000000, 0x7050700000000000, 0xe0a0e00000000000, 0xc040c00000000000,
	0x0203000000000000, 0x0507000000000000, 0x0a0e000000000000, 0x141c000000000000,
	0x2838000000000000, 0x5070000000000000, 0xa0e0000000000000, 0x40c0000000000000,
];

#[rustfmt::skip]
const KNIGHT_MOVES_TABLE: [u64; Square::N] = [
    0x0000000000020400, 0x0000000000050800, 0x00000000000a1100, 0x0000000000142200,
	0x0000000000284400, 0x0000000000508800, 0x0000000000a01000, 0x0000000000402000,
	0x0000000002040004, 0x0000000005080008, 0x000000000a110011, 0x0000000014220022,
	0x0000000028440044, 0x0000000050880088, 0x00000000a0100010, 0x0000000040200020,
	0x0000000204000402, 0x0000000508000805, 0x0000000a1100110a, 0x0000001422002214,
	0x0000002844004428, 0x0000005088008850, 0x000000a0100010a0, 0x0000004020002040,
	0x0000020400040200, 0x0000050800080500, 0x00000a1100110a00, 0x0000142200221400,
	0x0000284400442800, 0x0000508800885000, 0x0000a0100010a000, 0x0000402000204000,
	0x0002040004020000, 0x0005080008050000, 0x000a1100110a0000, 0x0014220022140000,
	0x0028440044280000, 0x0050880088500000, 0x00a0100010a00000, 0x0040200020400000,
	0x0204000402000000, 0x0508000805000000, 0x0a1100110a000000, 0x1422002214000000,
	0x2844004428000000, 0x5088008850000000, 0xa0100010a0000000, 0x4020002040000000,
	0x0400040200000000, 0x0800080500000000, 0x1100110a00000000, 0x2200221400000000,
	0x4400442800000000, 0x8800885000000000, 0x100010a000000000, 0x2000204000000000,
	0x0004020000000000, 0x0008050000000000, 0x00110a0000000000, 0x0022140000000000,
	0x0044280000000000, 0x0088500000000000, 0x0010a00000000000, 0x0020400000000000,
];

#[rustfmt::skip]
const PAWN_ATTACKS_TABLE: [[u64; Square::N]; Color::N] = [
	[
		0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000,
		0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000,
		0x0000000000000002, 0x0000000000000005, 0x000000000000000a, 0x0000000000000014,
		0x0000000000000028, 0x0000000000000050, 0x00000000000000a0, 0x0000000000000040,
		0x0000000000000200, 0x0000000000000500, 0x0000000000000a00, 0x0000000000001400,
		0x0000000000002800, 0x0000000000005000, 0x000000000000a000, 0x0000000000004000,
		0x0000000000020000, 0x0000000000050000, 0x00000000000a0000, 0x0000000000140000,
		0x0000000000280000, 0x0000000000500000, 0x0000000000a00000, 0x0000000000400000,
		0x0000000002000000, 0x0000000005000000, 0x000000000a000000, 0x0000000014000000,
		0x0000000028000000, 0x0000000050000000, 0x00000000a0000000, 0x0000000040000000,
		0x0000000200000000, 0x0000000500000000, 0x0000000a00000000, 0x0000001400000000,
		0x0000002800000000, 0x0000005000000000, 0x000000a000000000, 0x0000004000000000,
		0x0000020000000000, 0x0000050000000000, 0x00000a0000000000, 0x0000140000000000,
		0x0000280000000000, 0x0000500000000000, 0x0000a00000000000, 0x0000400000000000,
		0x0002000000000000, 0x0005000000000000, 0x000a000000000000, 0x0014000000000000,
		0x0028000000000000, 0x0050000000000000, 0x00a0000000000000, 0x0040000000000000,
	],
	[
		0x0000000000000200, 0x0000000000000500, 0x0000000000000a00, 0x0000000000001400,
		0x0000000000002800, 0x0000000000005000, 0x000000000000a000, 0x0000000000004000,
		0x0000000000020000, 0x0000000000050000, 0x00000000000a0000, 0x0000000000140000,
		0x0000000000280000, 0x0000000000500000, 0x0000000000a00000, 0x0000000000400000,
		0x0000000002000000, 0x0000000005000000, 0x000000000a000000, 0x0000000014000000,
		0x0000000028000000, 0x0000000050000000, 0x00000000a0000000, 0x0000000040000000,
		0x0000000200000000, 0x0000000500000000, 0x0000000a00000000, 0x0000001400000000,
		0x0000002800000000, 0x0000005000000000, 0x000000a000000000, 0x0000004000000000,
		0x0000020000000000, 0x0000050000000000, 0x00000a0000000000, 0x0000140000000000,
		0x0000280000000000, 0x0000500000000000, 0x0000a00000000000, 0x0000400000000000,
		0x0002000000000000, 0x0005000000000000, 0x000a000000000000, 0x0014000000000000,
		0x0028000000000000, 0x0050000000000000, 0x00a0000000000000, 0x0040000000000000,
		0x0200000000000000, 0x0500000000000000, 0x0a00000000000000, 0x1400000000000000,
		0x2800000000000000, 0x5000000000000000, 0xa000000000000000, 0x4000000000000000,
		0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000,
		0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000,
	],
];
