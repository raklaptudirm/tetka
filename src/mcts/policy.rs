use ataxx::{BitBoard, Move};

pub type Fn = fn(position: &ataxx::Position, mov: Move) -> f64;

pub fn handcrafted(position: &ataxx::Position, mov: Move) -> f64 {
    let mut score = 0.0;

    let stm = position.side_to_move;
    let xtm = !position.side_to_move;

    let friends = position.bitboard(stm);
    let enemies = position.bitboard(xtm);

    let old_neighbours = BitBoard::single(mov.source());
    let new_neighbours = BitBoard::single(mov.target());

    score += (enemies & new_neighbours).cardinality() as f64 * 1.0;
    score += (friends & new_neighbours).cardinality() as f64 * 0.4;

    if mov.is_single() {
        score += 0.7;
    } else {
        score -= (friends & old_neighbours).cardinality() as f64 * 0.4;
    }

    score.max(0.1)
}
