use ataxx::Position;
use uxi::{error, Bundle, Command, Flag, RunError};

use super::Context;
use crate::mcts;

// TODO: Move these macros into UXI

macro_rules! lock_mutable {
    ($bundle:ident > $ctx:ident => $($stmt:stmt;)*) => {
        let mut $ctx = $bundle.lock();
        $(
            $stmt
        )*
        drop($ctx);
    };
}

macro_rules! lock {
    ($bundle:ident > $ctx:ident => $($stmt:stmt;)*) => {
        let $ctx = $bundle.lock();
        $(
            $stmt
        )*
        drop($ctx);
    };
}

pub fn go() -> Command<Context> {
    Command::new(|bundle: Bundle<Context>| {
        lock! {
            bundle > ctx =>
            let position = ctx.position; // Get the position to search
            let mut searcher = ctx.searcher.clone(); // Get the previous search state
        }

        let mut nodes = 0;

        // Update the searcher with the new position and start searching.
        searcher.update_position(position);
        let bestmove = searcher.search(parse_limits(&bundle, &position)?, &mut nodes);

        println!("bestmove {}", bestmove);

        lock_mutable! {
            bundle > ctx =>
            // Push the new search state to the context.
            ctx.searcher = searcher;
        }

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

fn parse_limits(bundle: &Bundle<Context>, position: &Position) -> Result<mcts::Limits, RunError> {
    ////////////////////////////////////////////
    // Check which of the limit flags are set //
    ////////////////////////////////////////////

    // Standard Time Control flags
    let binc = bundle.is_flag_set("binc");
    let winc = bundle.is_flag_set("winc");
    let btime = bundle.is_flag_set("btime");
    let wtime = bundle.is_flag_set("wtime");

    // Other limit flags
    let depth = bundle.is_flag_set("depth");
    let nodes = bundle.is_flag_set("nodes");
    let movetime = bundle.is_flag_set("movetime");

    // Infinite flag
    let infinite = bundle.is_flag_set("infinite");

    ///////////////////////////////////////////////////////
    // Ensure that the given flag configuration is valid //
    ///////////////////////////////////////////////////////

    let std_tc = btime || wtime || binc || winc;

    // Either all or none of the standard time control flags must be set.
    if std_tc && !(btime && wtime && binc && winc) {
        return error!("bad flag set: missing standard time control flags");
    }

    // No other time control flags may be set alongside 'infinite'.
    if infinite && (std_tc || depth || nodes || movetime) {
        return error!("bad flag set: time control flags set alongside infinite");
    }

    // A little utility macro to parse the given flag into the required type.
    macro_rules! get_flag {
        ($name:expr) => {
            match bundle.get_single_flag($name) {
                Some(value) => Some(value.parse()?),
                None => None,
            }
        };
    }

    //////////////////////////////////////
    // Parse the provided search limits //
    //////////////////////////////////////

    Ok(mcts::Limits {
        maxnodes: get_flag!("nodes"),
        maxdepth: get_flag!("depth"),
        movetime: if std_tc {
            let (time, incr) = match position.side_to_move {
                ataxx::Piece::Black => ("btime", "binc"),
                ataxx::Piece::White => ("wtime", "winc"),
                _ => unreachable!(),
            };

            let time: u128 = get_flag!(time).unwrap();
            let incr: u128 = get_flag!(incr).unwrap();

            Some((time / 20 + incr / 2).max(1))
        } else {
            get_flag!("movetime")
        },
        movestogo: get_flag!("movestogo"),
    })
}
