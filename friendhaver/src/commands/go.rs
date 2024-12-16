use tetka::games::isolation::{self, Position};
use tetka::uxi::{error, Bundle, Command, Flag, RunError};

use crate::search;

use super::Context;

// TODO: Move these macros into UXI

macro_rules! lock {
    ($bundle:ident > $ctx:ident => $($stmt:stmt;)*) => {
        let $ctx = $bundle.lock();
        $(
            $stmt
        )*
        drop($ctx);
    };

    ($bundle:ident > mut $ctx:ident => $($stmt:stmt;)*) => {
        let mut $ctx = $bundle.lock();
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
        let limits = parse_limits(&bundle, &position)?;
        let bestmove = searcher.search(limits, &mut nodes);

        println!("bestmove {}", bestmove);

        lock! {
            bundle > mut ctx =>
            // Push the new search state to the context.
            ctx.searcher = searcher;
        }

        Ok(())
    })
    // Flags for reporting the current time situation.
    .flag("p2inc", Flag::Single)
    .flag("p1inc", Flag::Single)
    .flag("p2time", Flag::Single)
    .flag("p1time", Flag::Single)
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

fn parse_limits(bundle: &Bundle<Context>, position: &Position) -> Result<search::Limits, RunError> {
    ////////////////////////////////////////////
    // Check which of the limit flags are set //
    ////////////////////////////////////////////

    // Standard Time Control flags
    let p1inc = bundle.is_flag_set("p1inc");
    let p2inc = bundle.is_flag_set("p2inc");
    let p1time = bundle.is_flag_set("p1time");
    let p2time = bundle.is_flag_set("p2time");

    // Other limit flags
    let depth = bundle.is_flag_set("depth");
    let nodes = bundle.is_flag_set("nodes");
    let movetime = bundle.is_flag_set("movetime");

    // Infinite flag
    let infinite = bundle.is_flag_set("infinite");

    ///////////////////////////////////////////////////////
    // Ensure that the given flag configuration is valid //
    ///////////////////////////////////////////////////////

    let std_tc = p2time || p1time || p2inc || p1inc;
    let oth_tc = depth || nodes || movetime;

    // No other time control flags may be set alongside 'infinite'.
    if infinite && (std_tc || oth_tc) {
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

    ////////////////////////////////////////////
    // Parse the provided search/perft limits //
    ////////////////////////////////////////////

    Ok(search::Limits {
        maxnodes: get_flag!("nodes"),
        maxdepth: get_flag!("depth"),
        movetime: if std_tc {
            let (time, incr) = match position.side_to_move {
                isolation::Color::White => ("p1time", "p1inc"),
                isolation::Color::Black => ("p2time", "p2inc"),
            };

            let time: u128 = get_flag!(time).unwrap_or(0);
            let incr: u128 = get_flag!(incr).unwrap_or(0);

            Some((time / 20 + incr / 2).max(1))
        } else {
            get_flag!("movetime")
        },
        movestogo: get_flag!("movestogo"),
    })
}
