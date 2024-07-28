use crate::ataxx::Position;

/// perft is a function to walk the move generation tree of strictly legal moves
/// to count all the leaf nodes of a certain depth.
///
/// If `SPLIT` is set to `true`, the perft value contributed by each legal move
/// in the current position is displayed separately. If `BULK` is set to `true`,
/// a trick known as bulk-counting is used, which makes it significantly faster.
///
/// In perft, nodes are only counted at the end after the last make-move. Thus
/// "higher" terminal nodes (e.g. mate or stalemate) are not counted, instead
/// the number of move paths of a certain depth. Perft ignores draws by
/// repetition, by the fifty-move rule and by insufficient material.
pub fn perft<const SPLIT: bool, const BULK: bool>(position: Position, depth: u8) -> u64 {
    // Bulk counting if enabled. Instead of calling make move and perft for each
    // move at depth 1, just return the number of legal moves, which is equivalent.
    if BULK && depth == 1 {
        return position.count_moves() as u64;
    }

    // At depth 0, perft is defined to be 1.
    if depth == 0 {
        return 1;
    }

    let mut nodes: u64 = 0;
    let movelist = position.generate_moves();

    // MoveList implements IntoIterator, so it should be possible to use it
    // directly in the for loop, but manual iterations seems to be faster.
    for i in 0..movelist.len() {
        let m = movelist[i];

        // Find the next position without updating the Hash, which is unnecessary
        // inside perft given uniquely identifying positions here is unnecessary.
        let new_position = position.after_move::<false>(m);

        // Spilt should always be disabled for child perft calls, and a child perft
        // should have the same bulk counting behavior as the parent perft call.
        let new_nodes = perft::<false, BULK>(new_position, depth - 1);

        // If spilt perft is enabled, print the nodes added due to this move.
        if SPLIT {
            println!("{}: {}", m, new_nodes);
        }

        // Add the new node count to the cumulative total.
        nodes += new_nodes;
    }

    nodes
}
