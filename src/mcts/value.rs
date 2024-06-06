pub type Fn = fn(position: &ataxx::Position) -> f64;

pub fn material(position: &ataxx::Position) -> f64 {
    let stm = position.side_to_move;

    let stm_piece_n = position.bitboard(stm).cardinality();
    let xtm_piece_n = position.bitboard(!stm).cardinality();

    let eval = stm_piece_n as f64 - xtm_piece_n as f64;

    1.0 / (1.0 + f64::exp(-eval / 400.0))
}
