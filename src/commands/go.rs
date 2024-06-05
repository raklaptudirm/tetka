use std::{mem, time};

use uxi::{error, Bundle, Command, Flag, RunError};

use super::Context;
use crate::mcts::{self, Node};

pub fn go() -> Command<Context> {
    Command::new(|bundle: Bundle<Context>| {
        let binc = bundle.is_flag_set("binc");
        let winc = bundle.is_flag_set("winc");
        let btime = bundle.is_flag_set("btime");
        let wtime = bundle.is_flag_set("wtime");

        let depth = bundle.is_flag_set("depth");
        let nodes = bundle.is_flag_set("nodes");
        let movetime = bundle.is_flag_set("movetime");

        let infinite = bundle.is_flag_set("infinite");

        let std_tc = btime || wtime || binc || winc;

        if std_tc && !(btime && wtime && binc && winc) {
            return error!("bad flag set: missing standard time control flags");
        }

        if infinite && (std_tc || depth || nodes || movetime) {
            return error!("bad flag set: time control flags set alongside infinite");
        }

        let ctx = bundle.lock();
        let position = ctx.position;
        drop(ctx);

        let limits = Limits {
            nodes: match bundle.get_single_flag("nodes") {
                Some(nodes) => nodes.parse()?,
                None => usize::MAX,
            },
            depth: match bundle.get_single_flag("depth") {
                Some(depth) => depth.parse()?,
                None => usize::MAX,
            },
            movetime: {
                let movetime = match bundle.get_single_flag("movetime") {
                    Some(movetime) => movetime.parse()?,
                    None => u128::MAX,
                };

                if std_tc {
                    macro_rules! get_flag {
                        ($name:literal) => {
                            bundle.get_single_flag($name).unwrap().parse()?
                        };
                    }

                    let our_time: u128 = match position.side_to_move {
                        ataxx::Piece::Black => get_flag!("wtime"),
                        ataxx::Piece::White => get_flag!("btime"),
                        _ => unreachable!(),
                    };

                    let our_inc: u128 = match position.side_to_move {
                        ataxx::Piece::Black => get_flag!("winc"),
                        ataxx::Piece::White => get_flag!("binc"),
                        _ => unreachable!(),
                    };

                    let usable_time = (our_time / 20 + our_inc / 2).max(1);

                    usable_time.min(movetime)
                } else {
                    movetime
                }
            },

            movestogo: match bundle.get_single_flag("movestogo") {
                Some(string) => Some(string.parse()?),
                None => None,
            },
        };

        let mut nodes = 0;
        let bestmove = search(position, limits, &mut nodes);

        println!("bestmove {}", bestmove);

        Ok(())
    })
    // Flags for reporting the current time situation.
    .flag("binc", Flag::Single)
    .flag("winc", Flag::Single)
    .flag("btime", Flag::Single)
    .flag("wtime", Flag::Single)
    .flag("movestogo", Flag::Single)
    // Flags for imposing other search limits.
    .flag("depth", Flag::Single)
    .flag("nodes", Flag::Single)
    .flag("movetime", Flag::Single)
    // Flags for setting the search type.
    // .flag("ponder", Flag::Single)
    .flag("infinite", Flag::Single)
    // This command should be run in a separate thread so that the Client
    // can still respond to and run other Commands while this one is running.
    .parallelize(true)
}

#[derive(Debug)]
pub struct Limits {
    pub depth: usize,
    pub nodes: usize,
    pub movetime: u128,
    pub movestogo: Option<usize>,
}

pub fn search(position: ataxx::Position, limits: Limits, total_nodes: &mut u64) -> ataxx::Move {
    let mut tree = mcts::Tree::new(position);
    let start = time::Instant::now();

    let mut depth = 0;
    let mut seldepth = 0;
    let mut cumulative_depth = 0;

    loop {
        let mut new_depth = 0;
        let node = tree.playout(&mut new_depth);

        cumulative_depth += new_depth;
        if new_depth > seldepth {
            seldepth = new_depth;
        }

        let avg_depth = cumulative_depth / tree.playouts();
        if avg_depth > depth {
            depth = avg_depth;

            let node = tree.node(node);

            // Make a new info report.
            println!(
                "info depth {} seldepth {} score cp {:.0} nodes {} nps {}",
                depth,
                seldepth,
                node.q() * 100.0,
                tree.playouts(),
                tree.nodes() * 1000 / start.elapsed().as_millis().max(1) as usize
            );
        }

        if tree.playouts() & 127 == 0 {
            if start.elapsed().as_millis() >= limits.movetime
                || depth >= limits.depth
                || tree.nodes() >= limits.nodes
            {
                break;
            }

            // Hard memory limit to prevent overuse.
            // TODO: Fix this by removing old nodes and stuff.
            if tree.nodes() > 2_000_000_000 / mem::size_of::<Node>() {
                break;
            }
        }
    }

    *total_nodes += tree.nodes() as u64;

    println!(
        "info depth {} seldepth {} score cp {:.0} nodes {} nps {}",
        cumulative_depth / tree.playouts(),
        seldepth,
        100.0,
        tree.playouts(),
        tree.nodes() * 1000 / start.elapsed().as_millis().max(1) as usize
    );

    // Verify the tree.
    // debug_assert!(tree.verify().is_ok());

    tree.best_move()
}
