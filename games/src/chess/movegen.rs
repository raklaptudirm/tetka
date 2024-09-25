use crate::interface::{
    BitBoardType, ColoredPieceType, MoveStore, PositionType,
};

use super::{
    moves, BitBoard, ColoredPiece, Direction, Move, MoveFlag, Piece, Position,
    Square,
};

pub struct MoveGenerationInfo<'a> {
    position: &'a Position,

    king: Square,

    friends: BitBoard,
    enemies: BitBoard,
    blocker: BitBoard,

    checkers: BitBoard,

    territory: BitBoard,

    checkmask: BitBoard,

    pinmask_l: BitBoard,
    pinmask_d: BitBoard,
}

impl<'a> MoveGenerationInfo<'a> {
    fn serialize<ML: MoveStore<Move>>(
        &self,
        source: Square,
        targets: BitBoard,
        movelist: &mut ML,
    ) {
        let targets = targets & self.checkmask & self.territory;
        for target in targets {
            movelist.push(Move::new(source, target, MoveFlag::Normal))
        }
    }

    fn serialize_towards<ML: MoveStore<Move>>(
        &self,
        offset: Direction,
        flag: MoveFlag,
        targets: BitBoard,
        movelist: &mut ML,
    ) {
        let targets = targets & self.checkmask & self.territory;
        for target in targets {
            movelist.push(Move::new(target.shift(-offset), target, flag))
        }
    }

    fn serialize_promotions<ML: MoveStore<Move>>(
        &self,
        offset: Direction,
        targets: BitBoard,
        movelist: &mut ML,
    ) {
        let targets = targets & self.checkmask & self.territory;
        for target in targets {
            movelist.push(Move::new(
                target.shift(-offset),
                target,
                MoveFlag::NPromotion,
            ));
            movelist.push(Move::new(
                target.shift(-offset),
                target,
                MoveFlag::BPromotion,
            ));
            movelist.push(Move::new(
                target.shift(-offset),
                target,
                MoveFlag::RPromotion,
            ));
            movelist.push(Move::new(
                target.shift(-offset),
                target,
                MoveFlag::QPromotion,
            ));
        }
    }
}

impl<'a> MoveGenerationInfo<'a> {
    fn generate_checkers(position: &Position, king: Square) -> BitBoard {
        let stm = position.side_to_move();
        let xtm = !stm;

        let friends = position.color_bb(stm);
        let enemies = position.color_bb(xtm);
        let blocker = friends | enemies;

        let p = position.piece_bb(Piece::Pawn);
        let n = position.piece_bb(Piece::Knight);
        let b = position.piece_bb(Piece::Bishop);
        let r = position.piece_bb(Piece::Rook);
        let q = position.piece_bb(Piece::Queen);

        let checking_p = p & moves::pawn_attacks(king, stm);
        let checking_n = n & moves::knight(king);
        let checking_b = (b | q) & moves::bishop(king, blocker);
        let checking_r = (r | q) & moves::rook(king, blocker);

        (checking_p | checking_n | checking_b | checking_r) & enemies
    }

    fn generate_checkmask(
        position: &Position,
        checkers: BitBoard,
        king: Square,
    ) -> BitBoard {
        match checkers.len() {
            0 => BitBoard::UNIVERSE,
            2 => BitBoard::EMPTY,
            _ => {
                let checker_sq =
                    unsafe { checkers.clone().next().unwrap_unchecked() };

                let checker_pc = unsafe {
                    position.at(checker_sq).unwrap_unchecked().piece()
                };

                if checker_pc == Piece::Pawn || checker_pc == Piece::Knight {
                    checkers
                } else {
                    BitBoard::between2(king, checker_sq)
                }
            }
        }
    }

    fn generate_pinmask(
        position: &Position,
        pinners: BitBoard,
        king: Square,
    ) -> BitBoard {
        let friends = position.color_bb(position.side_to_move());
        let mut pinmask = BitBoard::EMPTY;

        for possible_pinner in pinners {
            let possible_pin = BitBoard::between2(king, possible_pinner);
            if (friends & possible_pin).len() == 1 {
                pinmask |= possible_pin;
            }
        }

        pinmask
    }

    fn generate_pinmasks(
        position: &Position,
        king: Square,
    ) -> (BitBoard, BitBoard) {
        let enemies = position.color_bb(!position.side_to_move());

        let b = enemies & position.piece_bb(Piece::Bishop);
        let r = enemies & position.piece_bb(Piece::Rook);
        let q = enemies & position.piece_bb(Piece::Queen);

        (
            Self::generate_pinmask(
                position,
                (r | q) & moves::rook(king, enemies),
                king,
            ),
            Self::generate_pinmask(
                position,
                (b | q) & moves::bishop(king, enemies),
                king,
            ),
        )
    }

    fn attacked(&self, sq: Square) -> bool {
        let stm = self.position.side_to_move();

        let p = self.friends & self.position.piece_bb(Piece::Pawn);
        let n = self.friends & self.position.piece_bb(Piece::Knight);
        let b = self.friends & self.position.piece_bb(Piece::Bishop);
        let r = self.friends & self.position.piece_bb(Piece::Rook);
        let q = self.friends & self.position.piece_bb(Piece::Queen);

        !(p.is_disjoint(moves::pawn_attacks(sq, !stm))
            && n.is_disjoint(moves::knight(sq))
            && (b | q).is_disjoint(moves::bishop(sq, self.blocker))
            && (r | q).is_disjoint(moves::rook(sq, self.blocker)))
    }
}

impl<'a> MoveGenerationInfo<'a> {
    fn pawn_moves<ML: MoveStore<Move>>(&self, movelist: &mut ML) {
        let up = Direction::up(self.position.side_to_move());
        let ue = up + Direction::East;
        let uw = up + Direction::West;

        let pawns = self.position.piece_bb(Piece::Pawn) & self.friends;

        {
            let attackers = pawns - self.pinmask_l;

            let pinned_attackers = attackers & self.pinmask_d;
            let unpinned_attackers = attackers ^ pinned_attackers;

            let pinned_attacks_east = pinned_attackers.shift(ue);
            let pinned_attacks_west = pinned_attackers.shift(uw);
            let unpinned_attacks_east = unpinned_attackers.shift(ue);
            let unpinned_attacks_west = unpinned_attackers.shift(uw);

            let attacks_east =
                (pinned_attacks_east & self.pinmask_d) | unpinned_attacks_east;
            let attacks_west =
                (pinned_attacks_west & self.pinmask_d) | unpinned_attacks_west;

            self.serialize_towards(
                ue,
                MoveFlag::Normal,
                attacks_east & self.enemies,
                movelist,
            );
            self.serialize_towards(
                uw,
                MoveFlag::Normal,
                attacks_west & self.enemies,
                movelist,
            );
        }

        {
            let pushers = pawns - self.pinmask_d;

            let pinned_pushers = pushers & self.pinmask_l;
            let unpinned_pushers = pushers ^ pinned_pushers;

            let pinned_single_push = pinned_pushers.shift(up) - self.blocker;
            let unpinned_single_push =
                unpinned_pushers.shift(up) - self.blocker;

            let single_pushes =
                (pinned_single_push & self.pinmask_l) | unpinned_single_push;

            self.serialize_towards(
                up,
                MoveFlag::Normal,
                single_pushes,
                movelist,
            );
        }
    }

    fn knight_moves<ML: MoveStore<Move>>(&self, movelist: &mut ML) {
        let knights = (self.position.piece_bb(Piece::Knight) & self.friends)
            - (self.pinmask_l | self.pinmask_d);
        for knight in knights {
            self.serialize(knight, moves::knight(knight), movelist)
        }
    }

    fn bishop_moves<ML: MoveStore<Move>>(&self, movelist: &mut ML) {
        let bishops = ((self.position.piece_bb(Piece::Bishop)
            | self.position.piece_bb(Piece::Queen))
            & self.friends)
            - self.pinmask_l;

        let pinned = bishops & self.pinmask_d;
        for bishop in pinned {
            self.serialize(
                bishop,
                moves::bishop(bishop, self.blocker) & self.pinmask_d,
                movelist,
            )
        }

        let unpinned = bishops ^ pinned;
        for bishop in unpinned {
            self.serialize(
                bishop,
                moves::bishop(bishop, self.blocker),
                movelist,
            )
        }
    }

    fn rook_moves<ML: MoveStore<Move>>(&self, movelist: &mut ML) {
        let rooks = ((self.position.piece_bb(Piece::Rook)
            | self.position.piece_bb(Piece::Queen))
            & self.friends)
            - self.pinmask_d;

        let pinned = rooks & self.pinmask_l;
        for rook in pinned {
            self.serialize(
                rook,
                moves::rook(rook, self.blocker) & self.pinmask_l,
                movelist,
            )
        }

        let unpinned = rooks ^ pinned;
        for rook in unpinned {
            self.serialize(rook, moves::rook(rook, self.blocker), movelist)
        }
    }

    fn king_moves<ML: MoveStore<Move>>(&self, movelist: &mut ML) {
        let targets = moves::king(self.king) & self.territory;

        for target in targets {
            if !self.attacked(target) {
                movelist.push(Move::new(self.king, target, MoveFlag::Normal))
            }
        }
    }
}

impl<'a> MoveGenerationInfo<'a> {
    pub fn new(position: &'a Position) -> Self {
        let king = unsafe {
            position
                .colored_piece_bb(ColoredPiece::new(
                    Piece::King,
                    position.side_to_move(),
                ))
                .next()
                .unwrap_or_else(|| panic!("{}", position))
        };
        let checkers = Self::generate_checkers(position, king);
        let checkmask = Self::generate_checkmask(position, checkers, king);
        let pinmasks = Self::generate_pinmasks(position, king);

        let friends = position.color_bb(position.side_to_move());
        let enemies = position.color_bb(position.side_to_move());
        let blocker = friends | enemies;

        let territory = !friends;

        Self {
            position,
            king,
            friends,
            enemies,
            blocker,
            checkers,
            checkmask,
            territory,
            pinmask_l: pinmasks.0,
            pinmask_d: pinmasks.1,
        }
    }

    pub fn generate_moves_into<ML: MoveStore<Move>>(&self, movelist: &mut ML) {
        let checker_num = self.checkers.len();

        self.king_moves(movelist);

        if checker_num < 2 {
            self.pawn_moves(movelist);
            self.knight_moves(movelist);
            self.bishop_moves(movelist);
            self.rook_moves(movelist);
        }
    }
}
