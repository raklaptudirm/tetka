use tetka::games::{
    interface::{MoveType, PositionType},
    isolation,
};

#[derive(Clone)]
pub struct Searcher {
    position: isolation::Position,
}

impl Searcher {
    pub fn new(position: isolation::Position) -> Searcher {
        Searcher { position }
    }

    pub fn update_position(&mut self, position: isolation::Position) {
        self.position = position;
    }

    pub fn search(&mut self, limits: Limits, nodes: &mut usize) -> isolation::Move {
        let moves = self.position.generate_moves::<false, true, true>();
        moves.into_iter().next().unwrap_or(isolation::Move::NULL)
    }
}

#[derive(Debug)]
pub struct Limits {
    pub maxdepth: Option<usize>,
    pub maxnodes: Option<usize>,
    pub movetime: Option<u128>,

    #[allow(unused)]
    pub movestogo: Option<usize>,
}
