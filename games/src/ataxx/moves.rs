use super::{BitBoard, Square};
use crate::interface::{BitBoardType, RepresentableType};

pub fn singles(bb: BitBoard) -> BitBoard {
    let bar = bb | bb.east() | bb.west();
    bar | bar.north() | bar.south()
}

/// single returns the targets of a singular Move from the given Square.
/// ```
/// use ataxx::*;
///
/// assert_eq!(BitBoard::single(Square::A1), bitboard! {
///     . . . . . . .
///     . . . . . . .
///     . . . . . . .
///     . . . . . . .
///     . . . . . . .
///     X X . . . . .
///     . X . . . . .
/// });
/// ```
pub const fn single(square: Square) -> BitBoard {
    SINGLES[square as usize]
}

/// double returns the targets of a jump Move from the given Square.
/// ```
/// use ataxx::*;
///
/// assert_eq!(BitBoard::double(Square::A1), bitboard! {
///     . . . . . . .
///     . . . . . . .
///     . . . . . . .
///     . . . . . . .
///     X X X . . . .
///     . . X . . . .
///     . . X . . . .
/// });
/// ```
pub const fn double(square: Square) -> BitBoard {
    DOUBLES[square as usize]
}

const SINGLES: [BitBoard; Square::N] = [
    BitBoard(0x0000000000182),
    BitBoard(0x0000000000385),
    BitBoard(0x000000000070a),
    BitBoard(0x0000000000e14),
    BitBoard(0x0000000001c28),
    BitBoard(0x0000000003850),
    BitBoard(0x0000000003020),
    BitBoard(0x000000000c103),
    BitBoard(0x000000001c287),
    BitBoard(0x000000003850e),
    BitBoard(0x0000000070a1c),
    BitBoard(0x00000000e1438),
    BitBoard(0x00000001c2870),
    BitBoard(0x0000000181060),
    BitBoard(0x0000000608180),
    BitBoard(0x0000000e14380),
    BitBoard(0x0000001c28700),
    BitBoard(0x0000003850e00),
    BitBoard(0x00000070a1c00),
    BitBoard(0x000000e143800),
    BitBoard(0x000000c083000),
    BitBoard(0x000003040c000),
    BitBoard(0x0000070a1c000),
    BitBoard(0x00000e1438000),
    BitBoard(0x00001c2870000),
    BitBoard(0x00003850e0000),
    BitBoard(0x000070a1c0000),
    BitBoard(0x0000604180000),
    BitBoard(0x0001820600000),
    BitBoard(0x0003850e00000),
    BitBoard(0x00070a1c00000),
    BitBoard(0x000e143800000),
    BitBoard(0x001c287000000),
    BitBoard(0x003850e000000),
    BitBoard(0x003020c000000),
    BitBoard(0x00c1030000000),
    BitBoard(0x01c2870000000),
    BitBoard(0x03850e0000000),
    BitBoard(0x070a1c0000000),
    BitBoard(0x0e14380000000),
    BitBoard(0x1c28700000000),
    BitBoard(0x1810600000000),
    BitBoard(0x0081800000000),
    BitBoard(0x0143800000000),
    BitBoard(0x0287000000000),
    BitBoard(0x050e000000000),
    BitBoard(0x0a1c000000000),
    BitBoard(0x1438000000000),
    BitBoard(0x0830000000000),
];
const DOUBLES: [BitBoard; Square::N] = [
    BitBoard(0x000000001c204),
    BitBoard(0x000000003c408),
    BitBoard(0x000000007c891),
    BitBoard(0x00000000f9122),
    BitBoard(0x00000001f2244),
    BitBoard(0x00000001e0408),
    BitBoard(0x00000001c0810),
    BitBoard(0x0000000e10204),
    BitBoard(0x0000001e20408),
    BitBoard(0x0000003e44891),
    BitBoard(0x0000007c89122),
    BitBoard(0x000000f912244),
    BitBoard(0x000000f020408),
    BitBoard(0x000000e040810),
    BitBoard(0x0000070810207),
    BitBoard(0x00000f102040f),
    BitBoard(0x00001f224489f),
    BitBoard(0x00003e448913e),
    BitBoard(0x00007c891227c),
    BitBoard(0x0000781020478),
    BitBoard(0x0000702040870),
    BitBoard(0x0003840810380),
    BitBoard(0x0007881020780),
    BitBoard(0x000f912244f80),
    BitBoard(0x001f224489f00),
    BitBoard(0x003e448913e00),
    BitBoard(0x003c081023c00),
    BitBoard(0x0038102043800),
    BitBoard(0x01c204081c000),
    BitBoard(0x03c408103c000),
    BitBoard(0x07c891227c000),
    BitBoard(0x0f912244f8000),
    BitBoard(0x1f224489f0000),
    BitBoard(0x1e040811e0000),
    BitBoard(0x1c081021c0000),
    BitBoard(0x0102040e00000),
    BitBoard(0x0204081e00000),
    BitBoard(0x0448913e00000),
    BitBoard(0x0891227c00000),
    BitBoard(0x112244f800000),
    BitBoard(0x020408f000000),
    BitBoard(0x040810e000000),
    BitBoard(0x0102070000000),
    BitBoard(0x02040f0000000),
    BitBoard(0x04489f0000000),
    BitBoard(0x08913e0000000),
    BitBoard(0x11227c0000000),
    BitBoard(0x0204780000000),
    BitBoard(0x0408700000000),
];
