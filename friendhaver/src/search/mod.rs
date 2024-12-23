use std::i16;

use tetka::games::{
    interface::{BitBoard, MoveType, PositionType},
    isolation::{self, Move, Piece, Position},
};

use derive_more::{Add, Neg, Sub};

#[derive(Clone)]
pub struct Searcher {
    position: isolation::Position,
}

impl Searcher {
    pub fn new(position: isolation::Position) -> Searcher {
        Searcher { position }
    }

    pub fn update_position(&mut self, position: isolation::Position) {
        self.position = position;
    }

    pub fn search(&mut self, limits: Limits, nodes: &mut usize) -> isolation::Move {
        let moves = self.position.generate_moves::<false, true, true>();
        moves.into_iter().next().unwrap_or(isolation::Move::NULL)
    }
}

#[derive(Debug)]
pub struct Limits {
    pub maxdepth: Option<usize>,
    pub maxnodes: Option<usize>,
    pub movetime: Option<u128>,

    #[allow(unused)]
    pub movestogo: Option<usize>,
}

#[derive(Clone, Copy, Add, Sub, Neg, PartialEq, PartialOrd)]
pub struct Score(i16);

impl Score {
    const INF: Score = Score(i16::MAX - 1);
    const NIL: Score = Score(i16::MAX);
    const MAX_TILL_MATE: Score = Score(i16::MAX - 48);
}

impl Searcher {
    fn negamax(
        &mut self,
        position: isolation::Position,
        mut alpha: Score,
        beta: Score,
        depth: u8,
    ) -> Score {
        if depth == 0 {
            return Searcher::evaluate(&position);
        }

        let moves = position.generate_moves::<true, true, true>();
        if moves.is_empty() {
            return -Score::INF;
        }

        let mut best_move = Move::NULL;
        let mut best_score = -Score::INF;

        for mov in moves.into_iter() {
            let new_position = position.after_move::<true>(mov);
            let score = -self.negamax(new_position, -beta, -alpha, depth - 1);

            if score > best_score {
                best_score = score;

                if score > alpha {
                    best_move = mov;
                    alpha = score;

                    if score > beta {
                        return beta;
                    }
                }
            }
        }

        best_score
    }

    fn evaluate(position: &Position) -> Score {
        let stm = position.side_to_move;

        Score(
            ((position.piece_bb(Piece::Tile)
                & isolation::BitBoard::singles(position.color_bb(stm)))
            .count()
                * 100) as i16,
        )
    }
}
