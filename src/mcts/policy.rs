use super::Node;
use ataxx::{BitBoard, Move, MoveStore};

pub type Fn = fn(node: &Node, mov: Move) -> f64;

pub fn handcrafted(node: &Node, mov: Move) -> f64 {
    let mut score = 0.0;

    let stm = node.position.side_to_move;
    let xtm = !node.position.side_to_move;

    let friends = node.position.bitboard(stm);
    let enemies = node.position.bitboard(xtm);

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
