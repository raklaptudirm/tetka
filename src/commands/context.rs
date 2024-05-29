use std::str::FromStr;

use ataxx::Position;

pub struct Context {
    pub position: Position,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            position: Position::from_str("x5o/7/7/7/7/7/o5x x 0 1").unwrap(),
        }
    }
}
