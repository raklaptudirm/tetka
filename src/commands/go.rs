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
        // let movestogo = bundle.is_flag_set("movestogo");
        let depth = bundle.is_flag_set("depth");
        let nodes = bundle.is_flag_set("nodes");
        let movetime = bundle.is_flag_set("movetime");
        // let ponder = bundle.is_flag_set("ponder");
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

        let movestogo = match bundle.get_single_flag("movestogo") {
            Some(string) => Some(string.parse()?),
            None => None,
        };

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

                    let tc = StandardTC {
                        btime: get_flag!("btime"),
                        wtime: get_flag!("wtime"),
                        binc: get_flag!("binc"),
                        winc: get_flag!("winc"),
                    };

                    let our_time = match position.side_to_move {
                        ataxx::Piece::Black => tc.wtime,
                        ataxx::Piece::White => tc.btime,
                        _ => unreachable!(),
                    };

                    let our_inc = match position.side_to_move {
                        ataxx::Piece::Black => tc.winc,
                        ataxx::Piece::White => tc.binc,
                        _ => unreachable!(),
                    };

                    let usable_time =
                        (our_time / movestogo.unwrap_or(20) as u128 + our_inc / 2).max(1);

                    usable_time.min(movetime)
                } else {
                    movetime
                }
            },

            movestogo,
        };

        let bestmove = search(position, limits);

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

#[allow(dead_code)]
pub struct StandardTC {
    pub btime: u128,
    pub wtime: u128,
    pub binc: u128,
    pub winc: u128,
}

#[derive(Debug)]
pub struct Limits {
    pub depth: usize,
    pub nodes: usize,
    pub movetime: u128,
    pub movestogo: Option<usize>,
}

pub fn search(position: ataxx::Position, limits: Limits) -> ataxx::Move {
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

    println!(
        "info depth {} seldepth {} score cp {:.0} nodes {} nps {}",
        cumulative_depth / tree.playouts(),
        seldepth,
        100.0,
        tree.playouts(),
        tree.nodes() * 1000 / start.elapsed().as_millis().max(1) as usize
    );

    // Verify the tree.
    debug_assert!(tree.verify().is_ok());

    tree.best_move()
}
