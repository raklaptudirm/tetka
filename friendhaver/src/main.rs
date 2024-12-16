use std::env;

use tetka::uxi::Client;

mod commands;
mod options;
mod search;

fn main() {
    let client = Client::new()
        .protocol("uii")
        .engine("FriendHaver v0.0.0")
        .author("Rak Laptudirm")
        // Register engine options.
        .option("Hash", options::hash())
        .option("Threads", options::threads())
        // Register the custom commands.
        .command("d", commands::d())
        .command("go", commands::go())
        .command("protocol", commands::protocol())
        .command("position", commands::position())
        .command("uginewgame", commands::uginewgame());

    let args = env::args()
        .skip(1)
        .reduce(|acc, e| format!("{} {}", acc, e))
        .unwrap_or("".to_string());
    let args = args.trim().to_string();
    if args.is_empty() {
        client.start(Default::default());
    } else {
        println!("args found {}", args);
        if let Err(err) = client.run_cmd_string(args, Default::default()) {
            println!("{}", err);
        };
    }
}
