use std::str::FromStr;

use tetka::games::isolation::Position;

use crate::search::{self, Searcher};

pub struct Context {
    pub position: Position,
    pub searcher: search::Searcher,
}

impl Default for Context {
    fn default() -> Self {
        let position =
            Position::from_str("--------/--------/p-------/-------P/--------/-------- w 1")
                .unwrap();
        Context {
            position,
            searcher: Searcher::new(position),
        }
    }
}
