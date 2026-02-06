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

use std::{cmp, fmt, num::ParseIntError, str::FromStr, sync::LazyLock};

use crate::interface::{
    parse::{self, FENParsablePosition, PiecePlacementParseError},
    BitBoardType, Hash, MoveStore, MoveType, PositionType, RepresentableType,
    SetType, SquareType, TypeParseError,
};

use strum::IntoEnumIterator;
use thiserror::Error;

// The Ataxx board has 7 Files and 7 Ranks, for a total of 49 Squares. It has
// two types of pieces, the Piece and the Blocker. Among the two only Piece is
// colored. The two colors are Black and White respectively, with Black moving
// first.
crate::interface::cartesian_square!(
    Files: A, B, C, D, E, F, G;
    Ranks: 1 First, 2 Second, 3 Third, 4 Fourth, 5 Fifth, 6 Sixth, 7 Seventh;
);
crate::interface::cartesian_piece!(
    Pieces: Piece "x"; Block "-";
    Colors: Black "x" ("x"),
            White "o" ("o");
);

/// Position represents the snapshot of an Ataxx Board, the state of the an
/// ataxx game at a single point in time. It also provides all of the methods
/// necessary to manipulate such a snapshot.
#[derive(Copy, Clone)]
pub struct Position {
    /// bitboards stores [BitBoard]s for the piece configuration of each piece.
    pub bitboards: [BitBoard; ColoredPiece::N],
    /// checksum stores the semi-unique [struct@Hash] of the current Position.
    pub checksum: Hash,
    /// side_to_move stores the piece whose turn to move it currently is.
    pub side_to_move: Color,
    pub ply_count: u16,
    /// half-move clock stores the number of half-moves since the last irreversible
    /// Move. It is used to adjudicate games using the 50-move/100-ply rule.
    pub half_move_clock: u8,
}

impl PositionType for Position {
    type Color = Color;
    type Move = Move;

    const STARTPOS: &str = "x5o/7/7/7/7/7/o5x x 0 1";

    fn ply_count(&self) -> usize {
        self.ply_count as usize
    }

    fn hash(&self) -> Hash {
        self.checksum
    }

    fn is_game_over(&self) -> bool {
        let black = self.colored_piece_bb(ColoredPiece::BlackPiece);
        let white = self.colored_piece_bb(ColoredPiece::WhitePiece);
        let block = self.colored_piece_bb(ColoredPiece::Block);

        self.half_move_clock >= 100 ||                           // Fifty-move rule
			white | black | block == BitBoard::UNIVERSE ||       // All squares occupied
			white == BitBoard::EMPTY || black == BitBoard::EMPTY // No pieces left
    }

    fn winner(&self) -> Option<Option<Color>> {
        if self.half_move_clock >= 100 {
            // Draw by 50 move rule.
            return None;
        }

        let black = self.colored_piece_bb(ColoredPiece::BlackPiece);
        let white = self.colored_piece_bb(ColoredPiece::WhitePiece);
        let block = self.colored_piece_bb(ColoredPiece::Block);

        if black == BitBoard::EMPTY {
            // Black lost all its pieces, White won.
            return Some(Some(Color::White));
        } else if white == BitBoard::EMPTY {
            // White lost all its pieces, Black won.
            return Some(Some(Color::Black));
        }

        debug_assert!(black | white | block == BitBoard::UNIVERSE);

        // All the squares are occupied by pieces. Victory is decided by
        // which Piece has the most number of pieces on the Board.

        let black_n = black.len();
        let white_n = white.len();

        match black_n.cmp(&white_n) {
            cmp::Ordering::Less => Some(Some(Color::White)),
            cmp::Ordering::Greater => Some(Some(Color::Black)),
            // Though there can't be an equal number of black and white pieces
            // on an empty ataxx board, it is possible with an odd number of
            // blocker pieces.
            cmp::Ordering::Equal => Some(None),
        }
    }

    fn after_move<const UPDATE_HASH: bool>(&self, m: Move) -> Position {
        let stm = self.side_to_move;

        macro_rules! update_hash {
            ($e:expr) => {
                if UPDATE_HASH {
                    $e
                } else {
                    Default::default()
                }
            };
        }

        if m == Move::PASS {
            return Position {
                bitboards: self.bitboards,
                checksum: update_hash!(!self.checksum),
                side_to_move: !self.side_to_move,
                ply_count: self.ply_count + 1,
                half_move_clock: self.half_move_clock + 1,
            };
        }

        let stm_pieces = self.color_bb(stm);
        let xtm_pieces = self.color_bb(!stm);

        let captured = BitBoard::single(m.target()) & xtm_pieces;
        let from_to = BitBoard::from(m.target()) | BitBoard::from(m.source());

        // Move the captured pieces from xtm to stm.
        let new_xtm = xtm_pieces ^ captured;
        let new_stm = stm_pieces ^ captured ^ from_to;

        // Reset half move clock on a singular move.
        let half_move_clock = if m.is_single() {
            0
        } else {
            self.half_move_clock + 1
        };

        let (white, black) = if stm == Color::White {
            (new_stm, new_xtm)
        } else {
            (new_xtm, new_stm)
        };

        Position {
            bitboards: [
                black,
                white,
                self.colored_piece_bb(ColoredPiece::Block),
            ],
            checksum: update_hash!(Self::get_hash(black, white, !stm)),
            side_to_move: !stm,
            ply_count: self.ply_count + 1,
            half_move_clock,
        }
    }

    fn generate_moves_into<
        const ALLOW_ILLEGAL: bool,
        const QUIET: bool,
        const NOISY: bool,
        T: MoveStore<Move>,
    >(
        &self,
        movelist: &mut T,
    ) {
        if self.is_game_over() {
            // Game is over, so don't generate any moves.
            return;
        }

        let stm = self.color_bb(self.side_to_move);
        let xtm = self.color_bb(!self.side_to_move);
        let gap = self.colored_piece_bb(ColoredPiece::Block);

        // Pieces can only move to unoccupied Squares.
        let allowed = !(stm | xtm | gap);

        for target in BitBoard::singles(stm) & allowed {
            movelist.push(Move::new_single(target));
        }

        for piece in stm {
            // There may be multiple jump moves to a single Square, so they need to be
            // verified (& allowed) and serialized into the movelist immediately.
            let double = BitBoard::double(piece) & allowed;
            for target in double {
                movelist.push(Move::new(piece, target));
            }
        }

        // If there are no legal moves possible on the Position and the game isn't
        // over, a pass move is the only move possible to be played.
        if movelist.len() == 0 {
            movelist.push(Move::PASS);
        }
    }

    fn count_moves<const QUIET: bool, const NOISY: bool>(&self) -> usize {
        if self.is_game_over() {
            // Game is over, so don't generate any moves.
            return 0;
        }

        let stm = self.color_bb(self.side_to_move);
        let xtm = self.color_bb(!self.side_to_move);
        let gap = self.colored_piece_bb(ColoredPiece::Block);

        // Pieces can only move to unoccupied Squares.
        let allowed = !(stm | xtm | gap);

        // Count the number single moves in the Position.
        let mut moves: usize = (BitBoard::singles(stm) & allowed).len();

        for piece in stm {
            // There may be multiple jump moves to a single Square, so they need to be
            // verified (& allowed) and counted into the Position total immediately.
            let double = BitBoard::double(piece) & allowed;
            moves += double.len();
        }

        // If there are no legal moves possible on the Position and the game isn't
        // over, a pass move is the only move possible to be played.
        if moves == 0 {
            return 1;
        }

        moves
    }
}

impl FENParsablePosition for Position {
    type Square = Square;
    type ColoredPiece = ColoredPiece;

    fn insert(&mut self, sq: Square, piece: ColoredPiece) {
        self.bitboards[piece].insert(sq);
    }

    fn remove(&mut self, sq: Square) -> Option<ColoredPiece> {
        match self.at(sq) {
            Some(piece) => {
                self.bitboards[piece].remove(sq);
                Some(piece)
            }
            None => None,
        }
    }

    fn at(&self, sq: Square) -> Option<ColoredPiece> {
        ColoredPiece::iter()
            .find(|piece| self.colored_piece_bb(*piece).contains(sq))
    }
}

impl Position {
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn half_move_clock(&self) -> usize {
        self.half_move_clock as usize
    }

    pub fn piece_bb(&self, piece: Piece) -> BitBoard {
        self.bitboards[piece]
    }

    pub fn color_bb(&self, color: Color) -> BitBoard {
        self.bitboards[color]
    }

    pub fn colored_piece_bb(&self, piece: ColoredPiece) -> BitBoard {
        self.bitboards[piece]
    }
    fn get_hash(black: BitBoard, white: BitBoard, stm: Color) -> Hash {
        let a = black.into();
        let b = white.into();

        // Currently, an 2^-63-almost delta universal hash function, based on
        // https://eprint.iacr.org/2011/116.pdf by Long Hoang Nguyen and Andrew
        // William Roscoe is used to create the Hash. This may change in the future.

        // 3 64-bit integer constants used in the hash function.
        const X: u64 = 6364136223846793005;
        const Y: u64 = 1442695040888963407;
        const Z: u64 = 2305843009213693951;

        // xa + yb + floor(ya/2^64) + floor(zb/2^64)
        // floor(pq/2^64) is essentially getting the top 64 bits of p*q.
        let part_1 = X.wrapping_mul(a); // xa
        let part_2 = Y.wrapping_mul(b); // yb
        let part_3 = (Y as u128 * a as u128) >> 64; // floor(ya/2^64) = ya >> 64
        let part_4 = (Z as u128 * b as u128) >> 64; // floor(zb/2^64) = zb >> 64

        // add the parts together and return the resultant hash.
        let hash = part_1
            .wrapping_add(part_2)
            .wrapping_add(part_3 as u64)
            .wrapping_add(part_4 as u64);

        // The Hash is bitwise complemented if the given side to move is Black.
        // Therefore, if two Positions only differ in side to move,
        // `a.Hash == !b.Hash`.
        if stm == Color::Black {
            Hash::new(!hash)
        } else {
            Hash::new(hash)
        }
    }
}

/// PositionParseErr represents an error encountered while parsing
/// the given FEN position field into a valid Position.
#[derive(Error, Debug)]
pub enum PositionParseError {
    #[error("expected 3 fields, found {0}")]
    TooManyFields(usize),

    #[error("parsing piece placement: {0}")]
    BadPiecePlacement(#[from] PiecePlacementParseError),

    #[error("parsing side to move: {0}")]
    BadSideToMove(#[from] TypeParseError),
    #[error("parsing half-move clock: {0}")]
    BadHalfMoveClock(#[from] ParseIntError),
}

// FromStr implements parsing of the position field in a FEN.
impl FromStr for Position {
    type Err = PositionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect::<Vec<&str>>();

        if parts.len() != 4 {
            return Err(PositionParseError::TooManyFields(parts.len()));
        }

        let pos = parts[0];
        let stm = parts[1];
        let hmc = parts[2];
        let fmc = parts[3];

        let mut position = Position {
            bitboards: [BitBoard::EMPTY; ColoredPiece::N],
            checksum: Default::default(),
            side_to_move: Color::Black,
            ply_count: 0,
            half_move_clock: 0,
        };

        parse::piece_placement(&mut position, pos)?;

        position.side_to_move = Color::from_str(stm)?;
        position.half_move_clock = hmc.parse::<u8>()?;
        position.ply_count = parse::ply_count(fmc, position.side_to_move)?;

        // Calculate the Hash value for the Position.
        position.checksum = Self::get_hash(
            position.colored_piece_bb(ColoredPiece::BlackPiece),
            position.colored_piece_bb(ColoredPiece::WhitePiece),
            position.side_to_move,
        );

        Ok(position)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::from_str(Self::STARTPOS).unwrap()
    }
}

// Display implements displaying a Position using ASCII art.
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board = self;
        let mut string_rep = String::from(" ");

        for rank in Rank::iter().rev() {
            for file in File::iter() {
                let square = Square::new(file, rank);
                let square_str = match board.at(square) {
                    Some(piece) => format!("{} ", piece),
                    None => ". ".to_string(),
                };
                string_rep += &square_str;
            }

            // Append the rank marker.
            string_rep += &format!(" {} \n ", rank);
        }

        // Append the file markers.
        string_rep += "a b c d e f g\n";

        writeln!(f, "{}", string_rep).unwrap();
        writeln!(f, "Side To Move: {}", self.side_to_move)
    }
}

/// Move represents an Ataxx move which can be played on the Board.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Move(u16);

impl MoveType for Move {
    const NULL: Self = Move(1 << 15);
    const MAX_IN_GAME: usize = 256;
    const MAX_IN_POSITION: usize = 256;

    type Position = Position;
    type MoveParseError = MoveParseError;

    /// from_str converts the given string representation of a Move into a [Move].
    /// The formats supported are '0000' for a [Move::PASS], `<target>` for a
    /// singular Move, and `<source><target>` for a jump Move. For how `<source>`
    /// and `<target>` are parsed, take a look at
    /// [`Square::FromStr`](Square::from_str). This function can be treated as the
    /// inverse of the [`fmt::Display`] trait for [Move].
    /// ```
    /// # use tetka_games::games::ataxx::*;
    /// # use std::str::FromStr;
    /// #
    /// let pass = Move::PASS;
    /// let sing = Move::new_single(Square::A1);
    /// let jump = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(Move::from_str(&pass.to_string()).unwrap(), pass);
    /// assert_eq!(Move::from_str(&sing.to_string()).unwrap(), sing);
    /// assert_eq!(Move::from_str(&jump.to_string()).unwrap(), jump);
    /// ```
    fn from_str(
        move_str: &str,
        _: &Self::Position,
    ) -> Result<Self, Self::MoveParseError> {
        if move_str == "0000" {
            return Ok(Move::PASS);
        };

        if move_str.len() != 2 && move_str.len() != 4 {
            return Err(MoveParseError::BadLength(move_str.len()));
        }

        let source = &move_str[..2];
        let source = Square::from_str(source)?;

        if move_str.len() < 4 {
            return Ok(Move::new_single(source));
        }

        let target = &move_str[2..];
        let target = Square::from_str(target)?;

        Ok(Move::new(source, target))
    }

    /// Display formats the given Move in a human-readable manner. The format used
    /// for displaying jump moves is `<source><target>`, while a singular Move is
    /// formatted as `<target>`. For the formatting of `<source>` and `<target>`,
    /// refer to `Square::Display`. [`Move::NULL`] is  formatted as `null`, while
    /// [`Move::PASS`] is formatted as `0000`.
    /// ```
    /// # use tetka_games::games::ataxx::*;
    /// #
    /// let null = Move::NULL;
    /// let pass = Move::PASS;
    /// let sing = Move::new_single(Square::A1);
    /// let jump = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(null.to_string(), "null");
    /// assert_eq!(pass.to_string(), "0000");
    /// assert_eq!(sing.to_string(), "a1");
    /// assert_eq!(jump.to_string(), "a1a3");
    /// ```
    fn fmt(
        &self,
        _: &Self::Position,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        if *self == Move::NULL {
            write!(f, "null")
        } else if *self == Move::PASS {
            write!(f, "0000")
        } else if self.is_single() {
            write!(f, "{}", self.source())
        } else {
            write!(f, "{}{}", self.source(), self.target())
        }
    }
}

impl From<u16> for Move {
    fn from(value: u16) -> Self {
        Move(value)
    }
}

impl From<Move> for u16 {
    fn from(value: Move) -> Self {
        value.0
    }
}

impl Move {
    // Bit-widths of fields.
    const SOURCE_WIDTH: u16 = 6;
    const TARGET_WIDTH: u16 = 6;

    // Bit-masks of fields.
    const SOURCE_MASK: u16 = (1 << Move::SOURCE_WIDTH) - 1;
    const TARGET_MASK: u16 = (1 << Move::TARGET_WIDTH) - 1;

    // Bit-offsets of fields.
    const SOURCE_OFFSET: u16 = 0;
    const TARGET_OFFSET: u16 = Move::SOURCE_OFFSET + Move::SOURCE_WIDTH;

    /// NULL Move represents an invalid move.
    pub const NULL: Move = Move(1 << 15);
    /// PASS Move represents a no move, where only the side to move changes.
    /// ```
    /// # use tetka_games::games::ataxx::*;
    /// # use tetka_games::interface::PositionType;
    /// # use std::str::FromStr;
    /// #
    /// let old_pos = Position::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    /// let new_pos = old_pos.after_move::<true>(Move::PASS);
    ///
    /// assert_eq!(old_pos.color_bb(Color::Black), new_pos.color_bb(Color::Black));
    /// assert_eq!(old_pos.color_bb(Color::White), new_pos.color_bb(Color::White));
    /// assert_eq!(old_pos.side_to_move, !new_pos.side_to_move);
    /// ```
    pub const PASS: Move = Move(1 << 15 | 1 << 14);

    /// new_single returns a new singular Move, where a piece is cloned to its
    /// target Square. For a singular Move, [`Move::source`] and [`Move::target`]
    /// are equal since the source Square is irrelevant to the Move.
    /// ```
    /// # use tetka_games::games::ataxx::*;
    /// #
    /// let mov = Move::new_single(Square::A1);
    ///
    /// assert_eq!(mov.source(), mov.target());
    /// assert_eq!(mov.target(), Square::A1);
    /// ```
    pub fn new_single(square: Square) -> Move {
        Move::new(square, square)
    }

    /// new returns a new jump Move from the given source Square to the given
    /// target Square. These Squares can be recovered with the [`Move::source`] and
    /// [`Move::target`] methods respectively.
    /// ```
    /// # use tetka_games::games::ataxx::*;
    /// #
    /// let mov = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(mov.source(), Square::A1);
    /// assert_eq!(mov.target(), Square::A3);
    /// ```
    #[rustfmt::skip]
    pub fn new(source: Square, target: Square) -> Move {
		Move(
			(source as u16) << Move::SOURCE_OFFSET |
			(target as u16) << Move::TARGET_OFFSET
		)
    }

    /// Source returns the source Square of the moving piece. This is equal to the
    /// target Square if the given Move is of singular type.
    /// ```
    /// # use tetka_games::games::ataxx::*;
    /// #
    /// let mov = Move::new(Square::A1, Square::A3);
    /// assert_eq!(mov.source(), Square::A1);
    /// ```
    pub fn source(self) -> Square {
        unsafe {
            Square::unsafe_from(
                (self.0 >> Move::SOURCE_OFFSET) & Move::SOURCE_MASK,
            )
        }
    }

    /// Target returns the target Square of the moving piece.
    /// ```
    /// # use tetka_games::games::ataxx::*;
    /// #
    /// let mov = Move::new(Square::A1, Square::A3);
    /// assert_eq!(mov.target(), Square::A3);
    /// ```
    pub fn target(self) -> Square {
        unsafe {
            Square::unsafe_from(
                (self.0 >> Move::TARGET_OFFSET) & Move::TARGET_MASK,
            )
        }
    }

    /// is_single checks if the given Move is singular in nature. The result of this
    /// function for [`Move::NULL`] and [`Move::PASS`] is undefined.
    /// ```
    /// # use tetka_games::games::ataxx::*;
    /// #
    /// let sing = Move::new_single(Square::A1);
    /// let jump = Move::new(Square::A1, Square::A3);
    ///
    /// assert!(sing.is_single());
    /// assert!(!jump.is_single());
    /// ```
    #[inline(always)]
    pub fn is_single(self) -> bool {
        self.source() == self.target()
    }
}

#[derive(Error, Debug)]
pub enum MoveParseError {
    #[error("length of move string should be 2 or 4, not {0}")]
    BadLength(usize),
    #[error("bad source square string \"{0}\"")]
    BadSquare(#[from] TypeParseError),
}

impl fmt::Debug for Move {
    /// Debug formats the given Move into a human-readable debug string. It uses
    /// `Move::Display` trait under the hood for formatting the Move.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Move::NULL {
            write!(f, "null")
        } else if *self == Move::PASS {
            write!(f, "0000")
        } else if self.is_single() {
            write!(f, "{}", self.source())
        } else {
            write!(f, "{}{}", self.source(), self.target())
        }
    }
}

impl BitBoard {
    /// singles returns the targets of all singular moves from all the source
    /// squares given in the provided BitBoard.
    pub fn singles(bb: BitBoard) -> BitBoard {
        let bar = bb | bb.east() | bb.west();
        bar | bar.north() | bar.south()
    }

    /// single returns the targets of a singular Move from the given Square.
    pub fn single(square: Square) -> BitBoard {
        SINGLES[square]
    }

    /// double returns the targets of a jump Move from the given Square.
    pub fn double(square: Square) -> BitBoard {
        DOUBLES[square]
    }
}

static SINGLES: LazyLock<[BitBoard; Square::N]> = LazyLock::new(|| {
    let mut singles = [BitBoard::EMPTY; Square::N];
    for square in Square::iter() {
        let square_bb = BitBoard::from(square);
        singles[square] = BitBoard::singles(square_bb) ^ square_bb;
    }
    singles
});

static DOUBLES: LazyLock<[BitBoard; Square::N]> = LazyLock::new(|| {
    let mut doubles = [BitBoard::EMPTY; Square::N];
    for square in Square::iter() {
        let singles = BitBoard::singles(BitBoard::from(square));
        doubles[square] = BitBoard::singles(singles) ^ singles;
    }
    doubles
});
