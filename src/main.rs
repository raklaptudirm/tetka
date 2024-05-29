use uxi::Client;

mod commands;
mod mcts;
mod options;

#[rustfmt::skip]
fn main() {
    Client::new()
        .protocol("uai")
        .engine("Mexx v0.0.0")
        .author("Rak Laptudirm")
        // Register engine options.
        .option("Hash",    options::hash   ())
        .option("Threads", options::threads())
        // Register the custom commands.
        .command(         "d", commands::d())
        .command(        "go", commands::go())
        .command(  "protocol", commands::protocol())
        .command(  "position", commands::position())
        .command("uainewgame", commands::uainewgame())
        // Start the Client so it can start running Commands.
        .start(Default::default());
}
