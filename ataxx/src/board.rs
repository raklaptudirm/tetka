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

use std::cmp;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use strum::IntoEnumIterator;

#[rustfmt::skip]
use crate::{
    BitBoard, Color, File, Hash, Move,
    MoveList, MoveStore, Rank, Square,
    ColorParseError,
};

/// Board implements an ataxx game board which can start from a given
/// Position and on which moves may be made and unmade to reach other
/// positions. It supports a maximum of MAX_PLY moves to be played.
pub struct Board {
    history: [Position; Board::MAX_PLY],
    current: usize,
    mov_cnt: u16,
}

impl Board {
    #[rustfmt::skip]
    pub fn new(position: Position, movcount: u16) -> Board {
        let mut board = Board {
            history: [
                Position::new(
                    BitBoard::EMPTY, BitBoard::EMPTY, Color::None
                ); Board::MAX_PLY
            ],
            current: 0,
            mov_cnt: movcount,
        };

        board.history[0] = position;
        board
    }
}

impl Board {
    /// MAX_PLY is the maximum number of plys than can be in a game.
    /// Actually there can be more plys but this is a nice upper bound to
    /// use for the length of the board's position history.
    const MAX_PLY: usize = 1024;

    /// position returns a copy of the current Position on the Board.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let board = Board::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    /// let position = Position::from_str("x5o/7/7/7/7/7/o5x x 0").unwrap();
    ///
    /// assert_eq!(board.checksum(), position.checksum);
    /// ```
    pub fn position(&self) -> Position {
        self.history[self.current]
    }

    /// side_to_move returns the current Color to move on the Board.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let board = Board::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    /// assert_eq!(board.side_to_move(), Color::Black);
    /// ```
    pub const fn side_to_move(&self) -> Color {
        self.current_pos().side_to_move
    }

    /// checksum returns a semi-unique Hash to identify the Position by.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let board = Board::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    ///
    /// assert_eq!(board.checksum(), board.position().checksum);
    /// ```
    pub const fn checksum(&self) -> Hash {
        self.current_pos().checksum
    }

    pub const fn full_moves(&self) -> u16 {
        self.mov_cnt
    }

    const fn current_pos(&self) -> &Position {
        &self.history[self.current]
    }
}

impl Board {
    ///
    pub const fn at(&self, sq: Square) -> Color {
        self.current_pos().at(sq)
    }

    ///
    pub const fn bitboard(&self, color: Color) -> BitBoard {
        self.current_pos().bitboard(color)
    }
}

impl Board {
    /// is_game_over checks if the game represented by the current Board has ended.
    /// It is a wrapper on top of [`Position::is_game_over`], so refer to it for
    /// more in-depth documentation and examples.
    pub fn is_game_over(&self) -> bool {
        self.current_pos().is_game_over()
    }

    /// winner returns the Color which has won the game represented by the current
    /// Board. It is a wrapper on top of [`Position::winner`], so refer to it for
    /// more in-depth documentation and examples.
    pub fn winner(&self) -> Color {
        self.current_pos().winner()
    }
}

impl Board {
    /// make_move plays the given Move on the Board and updates state accordingly.
    /// It is a wrapper on top of [`Position::after_move`], so refer to it for more
    /// in-depth documentation and examples.
    pub fn make_move(&mut self, m: Move) {
        self.history[self.current + 1] = self.current_pos().after_move(m);
        self.current += 1;
        if self.side_to_move() == Color::Black {
            self.mov_cnt += 1;
        }
    }

    /// undo_move un-plays the last played Move on the Board.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let mut board = Board::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    /// let old_checksum = board.position().checksum;
    ///
    /// board.make_move(Move::new_single(Square::B7));
    /// board.undo_move();
    ///
    /// assert_eq!(board.checksum(), old_checksum);
    /// ```
    pub fn undo_move(&mut self) {
        self.current -= 1;
    }
}

#[derive(Debug)]
pub enum BoardParseError {
    BadPosition(PositionParseError),
    BadMoveCount(ParseIntError),
}

/// Given a [PositionParseError] which was encountered while trying to parse the
/// Position part of the FEN string, it returns a [BoardParseError::BadPosition]
/// which wraps the given [PositionParseError].
impl From<PositionParseError> for BoardParseError {
    fn from(value: PositionParseError) -> Self {
        Self::BadPosition(value)
    }
}

/// Given a [ParseIntError] which was encountered while trying to parse the
/// move count in the FEN string, it returns a [BoardParseError::BadMoveCount]
/// which wraps the given [ParseIntError].
impl From<ParseIntError> for BoardParseError {
    fn from(value: ParseIntError) -> Self {
        Self::BadMoveCount(value)
    }
}

impl FromStr for Board {
    type Err = BoardParseError;

    /// from_str converts the given FEN string into a [Board].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split at the last space, the delimiter between the Position part and
        // the move count part of the given FEN string.
        let (pos, fm) = s.rsplit_once(' ').unwrap();

        let position = Position::from_str(pos)?; // Parse the Position
        let movcount = fm.parse::<u16>()?; // Parse the Move Count

        Ok(Board::new(position, movcount))
    }
}

impl Board {
    /// generate_moves generates the legal Moves present in the current Position
    /// on the Board into a [MoveList] and returns it. It is a wrapper on top of
    /// the [`Position::generate_moves`] function, so refer to it for more in
    /// depth documentation and examples.
    pub fn generate_moves(&self) -> MoveList {
        self.current_pos().generate_moves()
    }

    /// generate_moves_into generates the legal Moves present in the current
    /// Position into the given type which implements [MoveStore]. It is a
    /// wrapper on top of the [`Position::generate_moves_into<T>`] function, so
    /// refer to it for more in depth documentation and examples.
    pub fn generate_moves_into<T: MoveStore>(&self, movelist: &mut T) {
        self.current_pos().generate_moves_into(movelist);
    }

    /// count_moves counts the number of legal Moves present in the current
    /// Position. It is a wrapper on top of the [`Position::count_moves`]
    /// function, so refer to it for more in depth documentation and examples.
    pub fn count_moves(&self) -> usize {
        self.current_pos().count_moves()
    }
}

/// Position represents the snapshot of an Ataxx Board, the state of the Board
/// at a single point in time. It is the functional backend of the [Board] type.
#[derive(Copy, Clone)]
pub struct Position {
    /// bitboards stores [BitBoard]s for the piece configuration of each [Color].
    pub bitboards: [BitBoard; Color::N],
    /// checksum stores the semi-unique [struct@Hash] of the current Position.
    pub checksum: Hash,
    /// side_to_move stores the [Color] whose turn to move it currently is.
    pub side_to_move: Color,
    /// half-move clock stores the number of half-moves since the last irreversible
    /// Move. It is used to adjudicate games using the 50-move/100-ply rule.
    pub half_move_clock: u8,
}

impl Position {
    /// new creates a new Position with the given BitBoards and side to move.
    pub fn new(white: BitBoard, black: BitBoard, stm: Color) -> Position {
        Position {
            bitboards: [white, black],
            checksum: Hash::new(white, black, stm),
            side_to_move: stm,
            half_move_clock: 0,
        }
    }

    /// put puts the given piece represented by its Color on the given Square.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let mut position = Position::new(
    ///     BitBoard::EMPTY,
    ///     BitBoard::EMPTY,
    ///     Color::White
    /// );
    /// position.put(Square::A1, Color::White);
    ///
    /// assert_eq!(position.bitboard(Color::White), bitboard! {
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     . . . . . . .
    ///     X . . . . . .
    /// });
    /// ```
    pub fn put(&mut self, sq: Square, color: Color) {
        match color {
            Color::White => self.bitboards[Color::White as usize].insert(sq),
            Color::Black => self.bitboards[Color::Black as usize].insert(sq),
            Color::None => unreachable!(),
        };
    }

    /// at returns the Color of the piece present on the given Square.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let position = Position::new(
    ///     BitBoard::UNIVERSE,
    ///     BitBoard::EMPTY,
    ///     Color::White
    /// );
    /// assert_eq!(position.at(Square::A1), Color::White);
    /// ```
    pub const fn at(&self, sq: Square) -> Color {
        if self.bitboard(Color::White).contains(sq) {
            Color::White
        } else if self.bitboard(Color::Black).contains(sq) {
            Color::Black
        } else {
            Color::None
        }
    }

    /// bitboard returns the BitBoard associated to the piece configuration of the
    /// given Color. Only the Squares with a piece of the given Color on them are
    /// contained inside the returned BitBoard.
    /// ```
    /// use mexx::ataxx::*;
    ///
    /// let position = Position::new(
    ///     BitBoard::UNIVERSE,
    ///     BitBoard::EMPTY,
    ///     Color::White
    /// );
    /// assert_eq!(position.bitboard(Color::White), BitBoard::UNIVERSE);
    /// ```
    pub const fn bitboard(&self, color: Color) -> BitBoard {
        self.bitboards[color as usize]
    }
}

impl Position {
    /// is_game_over checks if the game is over, i.e. is a win or a draw.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let white_win = Position::from_str("ooooooo/7/7/7/7/7/7 x 0").unwrap();
    /// let black_win = Position::from_str("xxxxxxx/7/7/7/7/7/7 o 0").unwrap();
    /// let draw_game = Position::from_str("xxx1ooo/7/7/7/7/7/7 x 100").unwrap();
    /// let ongoing = Position::from_str("xxx1ooo/7/7/7/7/7/7 x 0").unwrap();
    ///
    /// assert!(white_win.is_game_over());
    /// assert!(black_win.is_game_over());
    /// assert!(draw_game.is_game_over());
    /// assert!(!ongoing.is_game_over());
    /// ```
    pub fn is_game_over(&self) -> bool {
        let white = self.bitboard(Color::White);
        let black = self.bitboard(Color::Black);
        self.half_move_clock >= 100 ||                           // Fifty-move rule
			white | black == BitBoard::UNIVERSE ||               // All squares occupied
			white == BitBoard::EMPTY || black == BitBoard::EMPTY // No pieces left
    }

    /// winner returns the Color which has won the game. It returns [`Color::None`]
    /// if the game is a draw. If [`Board::is_game_over`] is false, then the
    /// behavior of this function is undefined.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let white_win = Position::from_str("ooooooo/7/7/7/7/7/7 x 0").unwrap();
    /// let black_win = Position::from_str("xxxxxxx/7/7/7/7/7/7 o 0").unwrap();
    /// let draw_game = Position::from_str("xxx1ooo/7/7/7/7/7/7 x 100").unwrap();
    ///
    /// assert_eq!(white_win.winner(), Color::White);
    /// assert_eq!(black_win.winner(), Color::Black);
    /// assert_eq!(draw_game.winner(), Color::None);
    /// ```
    pub fn winner(&self) -> Color {
        debug_assert!(self.is_game_over());

        if self.half_move_clock >= 100 {
            // Draw by 50 move rule.
            return Color::None;
        }

        let white = self.bitboard(Color::White);
        let black = self.bitboard(Color::Black);

        if white == BitBoard::EMPTY {
            // White lost all its pieces, black won.
            return Color::Black;
        } else if black == BitBoard::EMPTY {
            // Black lost all its pieces, white won.
            return Color::White;
        }

        debug_assert!(white | black == BitBoard::UNIVERSE);

        // All the squares are occupied by pieces. Victory is decided by
        // which Color has the most number of pieces on the Board.

        let white_n = white.cardinality();
        let black_n = black.cardinality();

        match white_n.cmp(&black_n) {
            cmp::Ordering::Less => Color::Black,
            cmp::Ordering::Greater => Color::White,
            // Since there are an odd number of Squares and all of them are filled, there
            // can't be an equal number of black and white pieces on the Board.
            cmp::Ordering::Equal => unreachable!(),
        }
    }
}

impl Position {
    /// after_move returns a new Position which occurs when the given Move is
    /// played on the current Position. Its behavior is undefined if the given
    /// Move is illegal.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let mut pos = Position::from_str("x5o/7/7/7/7/7/o5x x 0").unwrap();
    /// let new_pos = Position::from_str("xx4o/7/7/7/7/7/o5x o 0").unwrap();
    ///
    /// let mov = Move::new_single(Square::B7);
    ///
    /// assert_eq!(pos.after_move(mov).checksum, new_pos.checksum);
    /// ```
    pub fn after_move(&self, m: Move) -> Position {
        let stm = self.side_to_move;

        // A pass move is a do nothing move; just change the side to move.
        if m == Move::PASS {
            return Position {
                bitboards: self.bitboards,
                checksum: Hash(!self.checksum.0),
                side_to_move: !self.side_to_move,
                half_move_clock: self.half_move_clock + 1,
            };
        }

        let stm_pieces = self.bitboard(stm);
        let xtm_pieces = self.bitboard(!stm);

        let captured = BitBoard::single(m.target()) & xtm_pieces;

        // Move the captured pieces from xtm to stm.
        let new_xtm = xtm_pieces ^ captured;
        let new_stm = stm_pieces ^ captured;

        // Add a stm piece to the target square.
        let mut new_stm = new_stm | BitBoard::from(m.target());

        // Reset half move clock on a singular move.
        let mut half_move_clock = 0;

        // Remove the piece from the source square if the move is non-singular.
        if !m.is_single() {
            new_stm ^= BitBoard::from(m.source());
            // Jump move, so don't reset half move clock.
            half_move_clock = self.half_move_clock + 1;
        };

        let (white, black) = if stm == Color::White {
            (new_stm, new_xtm)
        } else {
            (new_xtm, new_stm)
        };

        Position {
            bitboards: [white, black],
            checksum: Hash::new(white, black, !stm),
            side_to_move: !stm,
            half_move_clock,
        }
    }
}

impl Position {
    /// generate_moves generates the legal moves in the current Position and
    /// returns a [`MoveList`] containing all the moves. It is a wrapper on top of
    /// the more general [`Position::generate_moves_into<T>`].
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let position = Position::from_str("x5o/7/7/7/7/7/o5x x 0").unwrap();
    /// let movelist = position.generate_moves();
    ///
    /// // There are 16 possible moves in startpos.
    /// assert_eq!(movelist.len(), 16);
    /// ```
    pub fn generate_moves(&self) -> MoveList {
        let mut movelist = MoveList::new();
        self.generate_moves_into(&mut movelist);
        movelist
    }

    /// generate_moves_into generates all the legal moves in the current Position
    /// and adds them to the given movelist. The type of the movelist must
    /// implement the [`MoveStore`] trait.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let position = Position::from_str("x5o/7/7/7/7/7/o5x x 0").unwrap();
    /// let mut movelist = MoveList::new();
    ///
    /// position.generate_moves_into(&mut movelist);
    ///
    /// // There are 16 possible moves in startpos.
    /// assert_eq!(movelist.len(), 16);
    /// ```
    pub fn generate_moves_into<T: MoveStore>(&self, movelist: &mut T) {
        if self.is_game_over() {
            // Game is over, so don't generate any moves.
            return;
        }

        let stm = self.bitboard(self.side_to_move);
        let xtm = self.bitboard(!self.side_to_move);

        // Pieces can only move to unoccupied Squares.
        let allowed = !(stm | xtm);

        let mut single = BitBoard::EMPTY;
        for piece in stm {
            // All single moves to a single Square are equivalent, so a single
            // BitBoard is sufficient to keep track of them all and cast out duplicates.
            // An intersection with the allowed BitBoard is done once at the end.
            single |= BitBoard::single(piece);

            // There may be multiple jump moves to a single Square, so they need to be
            // verified (& allowed) and serialized into the movelist immediately.
            let double = BitBoard::double(piece) & allowed;
            for target in double {
                movelist.push(Move::new(piece, target));
            }
        }

        // Serialize the single moves into the movelist.
        single &= allowed;
        for target in single {
            movelist.push(Move::new_single(target));
        }

        // If there are no legal moves possible on the Position and the game isn't
        // over, a pass move is the only move possible to be played.
        if movelist.len() == 0 && !self.is_game_over() {
            movelist.push(Move::PASS);
        }
    }

    /// count_moves returns the number of legal moves in the current Position. It
    /// is faster than calling [`Position::generate_moves`] or
    ///  [`Position::generate_moves_into<T>`] and then finding the length.
    /// ```
    /// use mexx::ataxx::*;
    /// use std::str::FromStr;
    ///
    /// let position = Position::from_str("x5o/7/7/7/7/7/o5x x 0").unwrap();
    ///
    /// // There are 16 possible moves in startpos.
    /// assert_eq!(position.count_moves(), 16);
    /// ```
    pub fn count_moves(&self) -> usize {
        if self.is_game_over() {
            // Game is over, so don't generate any moves.
            return 0;
        }

        let mut moves: usize = 0;

        let stm = self.bitboard(self.side_to_move);
        let xtm = self.bitboard(!self.side_to_move);

        // Pieces can only move to unoccupied Squares.
        let allowed = !(stm | xtm);

        let mut single = BitBoard::EMPTY;
        for piece in stm {
            // All single moves to a single Square are equivalent, so a single
            // BitBoard is sufficient to keep track of them all and cast out duplicates.
            // An intersection with the allowed BitBoard is done once at the end.
            single |= BitBoard::single(piece);

            // There may be multiple jump moves to a single Square, so they need to be
            // verified (& allowed) and counted into the Position total immediately.
            let double = BitBoard::double(piece) & allowed;
            moves += double.cardinality();
        }

        // Count the number single moves in the Position.
        moves += (single & allowed).cardinality();

        // If there are no legal moves possible on the Position and the game isn't
        // over, a pass move is the only move possible to be played.
        if moves == 0 && !self.is_game_over() {
            moves += 1;
        }

        moves
    }
}

/// PositionParseErr represents an error encountered while parsing
/// the given FEN position field into a valid Position.
#[derive(Debug)]
pub enum PositionParseError {
    TooManyFields,

    /// A jump spec was too big for the current rank.
    JumpTooLong,
    /// Invalid character in Square spec.
    InvalidColorIdent,
    /// Insufficient number of Square spec entries.
    FileDataIncomplete,
    /// More fields in string spec than Ranks.
    TooManyRanks,

    BadSideToMove(ColorParseError),
    BadHalfMoveClock(ParseIntError),
}

impl From<ColorParseError> for PositionParseError {
    fn from(value: ColorParseError) -> Self {
        Self::BadSideToMove(value)
    }
}

impl From<ParseIntError> for PositionParseError {
    fn from(value: ParseIntError) -> Self {
        Self::BadHalfMoveClock(value)
    }
}

// FromStr implements parsing of the position field in a FEN.
impl FromStr for Position {
    type Err = PositionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect::<Vec<&str>>();

        if parts.len() != 3 {
            return Err(PositionParseError::TooManyFields);
        }

        let pos = parts[0];
        let stm = parts[1];
        let hmc = parts[2];

        let mut position = Position::new(BitBoard::EMPTY, BitBoard::EMPTY, Color::None);

        // Spilt the position spec by the Ranks which are separated by '/'.
        let ranks: Vec<&str> = pos.split('/').collect();

        let mut rank = Ok(Rank::Seventh);
        let mut file = Ok(File::A);

        // Iterate over the Ranks in the string spec.
        for rank_data in ranks {
            // Rank pointer ran out, but data carried on.
            if rank.is_err() {
                return Err(PositionParseError::TooManyRanks);
            }

            // Iterate over the Square specs in the Rank spec.
            for data in rank_data.chars() {
                // Check if a jump spec was too big and we landed on an invalid File.
                if file.is_err() {
                    return Err(PositionParseError::JumpTooLong);
                }
                let square = Square::new(file.unwrap(), rank.unwrap());
                match data {
                    'O' | 'o' | 'w' => position.put(square, Color::White),
                    'X' | 'x' | 'b' => position.put(square, Color::Black),

                    // Numbers represent jump specs to jump over empty squares.
                    '1'..='8' => {
                        file =
                            File::try_from(file.unwrap() as usize + data as usize - '1' as usize);
                        if file.is_err() {
                            return Err(PositionParseError::JumpTooLong);
                        }
                    }

                    _ => return Err(PositionParseError::InvalidColorIdent),
                }

                // On to the next Square spec in the Rank spec.
                file = File::try_from(file.unwrap() as usize + 1);
            }

            // After rank data runs out, file pointer should be
            // at the last file, i.e, rank is completely filled.
            if file.is_ok() {
                return Err(PositionParseError::FileDataIncomplete);
            }

            // Switch rank pointer and reset file pointer.
            rank = Rank::try_from((rank.unwrap() as usize).wrapping_sub(1));
            file = Ok(File::A);
        }

        position.side_to_move = Color::from_str(stm)?;
        position.half_move_clock = hmc.parse::<u8>()?;

        // Calculate the Hash value for the Position.
        position.checksum = Hash::new(
            position.bitboard(Color::White),
            position.bitboard(Color::Black),
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
                string_rep += &format!("{} ", board.at(square));
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
