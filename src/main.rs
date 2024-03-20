use mexx::ataxx::{Board, FEN, Move, Position, Square};
use std::str::FromStr;
use std::{env, time};
use std::sync::Arc;
use mexx::ataxx::zobrist::Hash;

#[derive(Copy, Clone)]
struct TTEntry {
    hash: Hash,
    depth: u8,
    nodes: usize,
}

const ENTRIES: usize = 1000000;//174762666;

struct Table {
    table: Vec<TTEntry>
}

impl Table {
    fn get(&mut self, hash: Hash) -> &mut TTEntry {
        let length = self.table.len();
        &mut self.table[hash.0 as usize % length]
    }
}

fn main() {
    let fen = FEN::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
    let mut board = Board::from(&fen);

    println!("{}\n", board.current_pos());

    let mut tt = Table{table: vec![TTEntry{hash: Hash(0), depth: 0, nodes: 0}; ENTRIES]};

    let depth = env::args().nth(1).unwrap();
    let depth = depth.parse::<u8>().unwrap();

    let start = time::Instant::now();
    let nodes = perft::<false, true>(&mut board, depth, &mut tt);
    let duration = start.elapsed();

    println!("nodes {} time {} nps {}", nodes, duration.as_millis(), (nodes as u128/duration.as_millis().max(1))*1000);
}

fn perft<const BULK: bool, const SPLIT: bool>(board: &mut Board, depth: u8, tt: &mut Table) -> usize {
    if BULK && depth == 1 {
        return board.count_moves();
    }

    if depth == 0 {
        return 1;
    }

    let mut nodes: usize = 0;
    let movelist = board.generate_moves();

    for m in movelist.iter() {
        board.make_move(m);
        // let hash = board.zobrist_hash();
        // let entry = tt.get(hash);
        //
        let new_nodes = //if entry.hash == hash && entry.depth == depth {
        //     entry.nodes
        // } else {
            perft::<BULK, false>(board, depth-1, tt)
        // }
        ;

        board.undo_move();

        if SPLIT {
            println!("{}: {}", m, new_nodes);
        }

        nodes += new_nodes;

        // let entry = tt.get(hash);
        // if depth >= 7 || entry.depth < depth {
        //     *entry = TTEntry {
        //         hash,
        //         depth,
        //         nodes: new_nodes,
        //     }
        // }
    }

    nodes
}
