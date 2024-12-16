use std::env;
use std::{str::FromStr, time};
use tetka::games::{isolation, perft};

fn main() {
    let depth = env::args().nth(1).unwrap_or("5".to_string());
    let fen = env::args()
        .nth(2)
        .unwrap_or("--------/--------/p-------/-------P/--------/-------- w 1".to_string());

    let position = isolation::Position::from_str(&fen).unwrap();
    println!("{}", position);

    let start = time::Instant::now();
    let nodes = perft::<true, true, isolation::Position>(position, depth.parse().unwrap());
    let elapsed = start.elapsed();

    println!(
        "nodes {} nps {}",
        nodes,
        nodes as u128 * 1000 / elapsed.as_millis().max(1)
    )
}
