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

use std::{fmt, num::ParseIntError, str::FromStr};

use crate::interface::{
    parse::{self, PiecePlacementParseError},
    BitBoardType, ColoredPieceType, Hash, MoveStore, MoveType, PositionType,
    RepresentableType, SetType, SquareType, TypeParseError,
};

use strum::IntoEnumIterator;
use thiserror::Error;

// The Isolation board has 8 Files and 6 Ranks, for a total of 48 Squares. It
// has two types of pieces, the Pawn and the Tile. Among the two only Pawn is
// colored. The two colors are White and Black respectively, with White moving
// first.
crate::interface::game_details!(
    Squares: u8 8 6;
    Pieces: Pawn "p"; Tile "-";
    Colors: White "w" ("P"),
            Black "b" ("p");
);

/// Position represents the snapshot of an Isolation Board, the state of the an
/// Isolation game at a single point in time. It also provides all of the methods
/// necessary to manipulate such a snapshot.
#[derive(Copy, Clone)]
pub struct Position {
    pub pawns: [Square; Color::N],
    pub tiles: BitBoard,
    /// checksum stores the semi-unique [struct@Hash] of the current Position.
    pub checksum: Hash,
    /// side_to_move stores the piece whose turn to move it currently is.
    pub side_to_move: Color,
    pub ply_count: u16,
}

impl PositionType for Position {
    type Square = Square;
    type ColoredPiece = ColoredPiece;
    type Move = Move;

    const STARTPOS: &str =
        "--------/--------/p-------/-------P/--------/-------- w 1";

    fn insert(&mut self, sq: Square, piece: ColoredPiece) {
        match piece.piece() {
            Piece::Pawn => self.set_pawn(piece.color(), sq),
            Piece::Tile => self.tiles.insert(sq),
        }
    }

    fn remove(&mut self, sq: Square) -> Option<ColoredPiece> {
        if self.pawn(Color::White) == sq {
            Some(ColoredPiece::WhitePawn)
        } else if self.pawn(Color::Black) == sq {
            Some(ColoredPiece::BlackPawn)
        } else if self.tiles.contains(sq) {
            self.tiles ^= BitBoard::from(sq);
            Some(ColoredPiece::Tile)
        } else {
            None
        }
    }

    fn at(&self, sq: Square) -> Option<ColoredPiece> {
        ColoredPiece::iter()
            .find(|piece| self.colored_piece_bb(*piece).contains(sq))
    }

    fn hash(&self) -> Hash {
        self.checksum
    }

    fn is_game_over(&self) -> bool {
        self.count_moves::<true, true>() == 0
    }

    fn winner(&self) -> Option<Option<Color>> {
        if self.is_game_over() {
            Some(Some(!self.side_to_move))
        } else {
            None
        }
    }

    fn after_move<const UPDATE_HASH: bool>(&self, m: Move) -> Position {
        let stm = self.side_to_move;

        let mut pawns = self.pawns;

        // Move our pawn to the new square.
        pawns[stm as usize] = m.pawn();

        // Remove the selected tile from the board.
        let tiles = self.colored_piece_bb(ColoredPiece::Tile)
            ^ BitBoard::from(m.tile());

        // Update the position checksum.
        let checksum = if UPDATE_HASH {
            Self::get_hash(pawns, tiles, !stm)
        } else {
            Default::default()
        };

        Position {
            pawns,
            tiles,
            checksum,
            side_to_move: !stm,
            ply_count: self.ply_count + 1,
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
        let stm = self.color_bb(self.side_to_move);
        let xtm = self.color_bb(!self.side_to_move);

        let tiles = self.colored_piece_bb(ColoredPiece::Tile);

        // Pieces can only move to unoccupied Squares.
        let allowed = tiles - xtm;

        for target in BitBoard::singles(stm) & allowed {
            for tile in allowed ^ BitBoard::from(target) {
                movelist.push(Move::new(target, tile));
            }
        }
    }

    fn count_moves<const QUIET: bool, const NOISY: bool>(&self) -> usize {
        let stm = self.color_bb(self.side_to_move);
        let xtm = self.color_bb(!self.side_to_move);

        let tiles = self.colored_piece_bb(ColoredPiece::Tile);

        // Pieces can only move to unoccupied Squares.
        let allowed = tiles - xtm;

        (BitBoard::singles(stm) & allowed).count() * (allowed.count() - 1)
    }

    fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    fn half_move_clock(&self) -> usize {
        0
    }

    fn ply_count(&self) -> usize {
        self.ply_count as usize
    }
}

impl Position {
    pub fn piece_bb(&self, piece: Piece) -> BitBoard {
        match piece {
            Piece::Pawn => BitBoard::from(self.pawns[0]) | self.pawns[1],
            Piece::Tile => self.tiles,
        }
    }

    pub fn color_bb(&self, color: Color) -> BitBoard {
        BitBoard::from(self.pawn(color))
    }

    pub fn colored_piece_bb(&self, piece: ColoredPiece) -> BitBoard {
        match piece.piece() {
            Piece::Pawn => BitBoard::from(self.pawn(piece.color())),
            Piece::Tile => self.tiles,
        }
    }
    fn pawn(&self, color: Color) -> Square {
        self.pawns[color as usize]
    }

    fn set_pawn(&mut self, color: Color, square: Square) {
        self.pawns[color as usize] = square
    }

    fn get_hash(
        pawns: [Square; Color::N],
        tiles: BitBoard,
        stm: Color,
    ) -> Hash {
        let a = pawns[0] as u64 * Square::N as u64 + pawns[1] as u64;
        let b = tiles.into();

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
    #[error("parsing full move count: {0}")]
    BadFullMoveCount(#[from] ParseIntError),
}

// FromStr implements parsing of the position field in a FEN.
impl FromStr for Position {
    type Err = PositionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect::<Vec<&str>>();

        if parts.len() != 3 {
            return Err(PositionParseError::TooManyFields(parts.len()));
        }

        let pos = parts[0];
        let stm = parts[1];
        let fmc = parts[2];

        let mut position = Position {
            pawns: [Square::default(), Square::default()],
            tiles: BitBoard::EMPTY,
            checksum: Default::default(),
            side_to_move: Color::Black,
            ply_count: 0,
        };

        parse::piece_placement(&mut position, pos)?;

        position.side_to_move = Color::from_str(stm)?;
        position.ply_count = parse::ply_count(fmc, position.side_to_move)?;

        // Calculate the Hash value for the Position.
        position.checksum = Self::get_hash(
            position.pawns,
            position.colored_piece_bb(ColoredPiece::Tile),
            position.side_to_move,
        );

        Ok(position)
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
        string_rep += "a b c d e f g h\n";

        writeln!(f, "{}", string_rep).unwrap();
        writeln!(f, "Side To Move: {}", self.side_to_move)
    }
}

/// Move represents an Isolation move which can be played on the Board.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Move(u16);

impl MoveType for Move {
    const NULL: Self = Move(1 << 15);
    const MAX_IN_GAME: usize = 48;
    const MAX_IN_POSITION: usize = 352;
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
    const PAWN_WIDTH: u16 = 6;
    const TILE_WIDTH: u16 = 6;

    // Bit-masks of fields.
    const PAWN_MASK: u16 = (1 << Move::PAWN_WIDTH) - 1;
    const TILE_MASK: u16 = (1 << Move::TILE_WIDTH) - 1;

    // Bit-offsets of fields.
    const PAWN_OFFSET: u16 = 0;
    const TILE_OFFSET: u16 = Move::PAWN_OFFSET + Move::PAWN_WIDTH;

    /// new returns a new jump Move from the given pawn Square to the given
    /// tile Square. These Squares can be recovered with the [`Move::pawn`] and
    /// [`Move::tile`] methods respectively.
    /// ```
    /// # use tetka_games::games::isolation::*;
    /// #
    /// let mov = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(mov.pawn(), Square::A1);
    /// assert_eq!(mov.tile(), Square::A3);
    /// ```
    #[rustfmt::skip]
    pub fn new(pawn: Square, tile: Square) -> Move {
		Move(
			(pawn as u16) << Move::PAWN_OFFSET |
			(tile as u16) << Move::TILE_OFFSET
		)
    }

    /// Source returns the pawn Square of the moving piece. This is equal to the
    /// tile Square if the given Move is of singular type.
    /// ```
    /// # use tetka_games::games::isolation::*;
    /// #
    /// let mov = Move::new(Square::A1, Square::A3);
    /// assert_eq!(mov.pawn(), Square::A1);
    /// ```
    pub fn pawn(self) -> Square {
        unsafe {
            Square::unsafe_from((self.0 >> Move::PAWN_OFFSET) & Move::PAWN_MASK)
        }
    }

    /// Target returns the tile Square of the moving piece.
    /// ```
    /// # use tetka_games::games::isolation::*;
    /// #
    /// let mov = Move::new(Square::A1, Square::A3);
    /// assert_eq!(mov.tile(), Square::A3);
    /// ```
    pub fn tile(self) -> Square {
        unsafe {
            Square::unsafe_from((self.0 >> Move::TILE_OFFSET) & Move::TILE_MASK)
        }
    }
}

#[derive(Error, Debug)]
pub enum MoveParseError {
    #[error("length of move string should be 2 or 4, not {0}")]
    BadLength(usize),
    #[error("bad pawn square string \"{0}\"")]
    BadSquare(#[from] TypeParseError),
}

impl FromStr for Move {
    type Err = MoveParseError;

    /// from_str converts the given string representation of a Move into a [Move].
    /// The format supported is `<pawn><tile>`. For how `<pawn>` and `<tile>` are
    /// parsed, take a look at [`Square::FromStr`](Square::from_str). This function
    /// can be treated as the inverse of the [`fmt::Display`] trait for [Move].
    /// ```
    /// # use tetka_games::games::isolation::*;
    /// # use std::str::FromStr;
    /// #
    /// let jump = Move::new(Square::A1, Square::A3);
    /// assert_eq!(Move::from_str(&jump.to_string()).unwrap(), jump);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(MoveParseError::BadLength(s.len()));
        }

        let pawn = Square::from_str(&s[..2])?;
        let tile = Square::from_str(&s[2..])?;

        Ok(Move::new(pawn, tile))
    }
}

impl fmt::Display for Move {
    /// Display formats the given Move in a human-readable manner. The format used
    /// for displaying moves is `<pawn><tile>`. For the formatting of `<pawn>` and
    /// `<tile>`, refer to `Square::Display`. [`Move::NULL`] is  formatted as `null`.
    /// ```
    /// # use tetka_games::games::isolation::*;
    /// # use tetka_games::interface::MoveType;
    /// #
    /// let null = Move::NULL;
    /// let jump = Move::new(Square::A1, Square::A3);
    ///
    /// assert_eq!(null.to_string(), "null");
    /// assert_eq!(jump.to_string(), "a1a3");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Move::NULL {
            write!(f, "null")
        } else {
            write!(f, "{}{}", self.pawn(), self.tile())
        }
    }
}

impl fmt::Debug for Move {
    /// Debug formats the given Move into a human-readable debug string. It uses
    /// `Move::Display` trait under the hood for formatting the Move.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl BitBoard {
    /// singles returns the targets of all singular moves from all the source
    /// squares given in the provided BitBoard.
    pub fn singles(bb: BitBoard) -> BitBoard {
        let bar = bb | bb.east() | bb.west();
        (bar | bar.north() | bar.south()) ^ bb
    }
}
