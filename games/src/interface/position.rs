use std::fmt::Display;
use std::str::FromStr;

use super::{BitBoardType, ColoredPieceType, MoveList, MoveStore, MoveType};

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
    fn generate_moves_into<const QUIET: bool, const NOISY: bool, T: MoveStore<Self::Move>>(
        &self,
        movelist: &mut T,
    );
    /// generate_moves is similar to generate_moves_into, except that instead of
    /// taking some storage as input it stores into a custom stack-based type.
    fn generate_moves<const QUIET: bool, const NOISY: bool>(&self) -> MoveList<Self::Move> {
        let mut movelist: MoveList<Self::Move> = Default::default();
        self.generate_moves_into::<QUIET, NOISY, _>(&mut movelist);
        movelist
    }
    /// count_moves is similar to generate_moves, except instead of returning a
    /// list of the available moves, it returns the number of available moves.
    /// By default this is simply `generate_moves().len()`, but implementations
    /// may take advantage of various optimizations counting as opposed to
    /// storing the moves allows to provide a more efficient version.
    fn count_moves<const QUIET: bool, const NOISY: bool>(&self) -> usize {
        self.generate_moves::<QUIET, NOISY>().len()
    }
}

#[macro_export]
macro_rules! position_type {
    (struct $name:tt {
        BitBoard = $bitboard_type:tt
        ColoredPiece = $piece_type:tt
        Move = $move_type:tt

        self = $my_self:tt

        fn winner $fn_winner:block
        fn is_game_over $fn_is_game_over:block
        fn after_move($arg_update:tt, $arg_move:tt) $fn_after_move:block
        fn generate_moves_into($arg_quiet:tt, $arg_noisy:tt, $arg_typ:tt, $arg_movelist:tt)
            $fn_generate_moves_into:block
        fn count_moves($arg_quiet_2:tt, $arg_noisy_2:tt) $fn_count_moves:block
    }) => {
        #[derive(Copy, Clone)]
        pub struct $name {
            /// bitboards stores [BitBoard]s for the piece configuration of each piece.
            pub bitboards: [BitBoard; $piece_type::N],
            /// checksum stores the semi-unique [struct@Hash] of the current Position.
            pub checksum: Hash,
            /// side_to_move stores the piece whose turn to move it currently is.
            pub side_to_move: <$piece_type as $crate::interface::ColoredPieceType>::Color,
            pub ply_count: u16,
            /// half-move clock stores the number of half-moves since the last irreversible
            /// Move. It is used to adjudicate games using the 50-move/100-ply rule.
            pub half_move_clock: u8,
        }

        impl $crate::interface::PositionType for $name {
            type BitBoard = $bitboard_type;
            type ColoredPiece = $piece_type;
            type Move = $move_type;

            /// put puts the given piece represented by its Piece on the given Square.
            fn insert(
                &mut self,
                sq: <$bitboard_type as $crate::interface::BitBoardType>::Square,
                piece: $piece_type,
            ) {
                self.bitboards[piece as usize].insert(sq);
            }

            fn remove(&mut self, sq: Square) -> Option<ColoredPiece> {
                match self.at(sq) {
                    Some(piece) => {
                        self.bitboards[piece as usize].remove(sq);
                        Some(piece)
                    }
                    None => None,
                }
            }

            fn at(&self, sq: Square) -> Option<ColoredPiece> {
                ColoredPiece::iter().find(|piece| self.colored_piece_bb(*piece).contains(sq))
            }

            fn piece_bb(
                &self,
                piece: <$piece_type as $crate::interface::ColoredPieceType>::Piece,
            ) -> BitBoard {
                self.bitboards[piece as usize]
            }

            fn color_bb(&self, color: Color) -> BitBoard {
                self.bitboards[color as usize]
            }

            fn colored_piece_bb(&self, piece: ColoredPiece) -> BitBoard {
                self.bitboards[piece as usize]
            }

            fn winner(&$my_self) -> Option<Color>
                $fn_winner

            fn is_game_over(&$my_self) -> bool
                $fn_is_game_over

            fn after_move<const $arg_update: bool>(&$my_self, $arg_move: Move) -> Self
                $fn_after_move

            fn generate_moves_into<
                const $arg_quiet: bool,
                const $arg_noisy: bool,
                $arg_typ: $crate::interface::MoveStore<Move>,
            >(
                &$my_self,
                $arg_movelist: &mut $arg_typ,
            )
                $fn_generate_moves_into

            fn count_moves<const $arg_quiet_2: bool, const $arg_noisy_2: bool>(&$my_self) -> usize
                $fn_count_moves
        }
    };
}
