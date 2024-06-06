pub mod policy;
pub mod value;

mod node;
mod params;
mod tree;

use std::mem;
use std::ops;
use std::time;

pub use self::node::*;
pub use self::params::*;
pub use self::tree::*;

#[derive(Clone)]
pub struct Searcher {
    tree: Tree,
    params: Params,
    policy: policy::Fn,
    value: value::Fn,
}

#[derive(Debug)]
pub struct Limits {
    pub maxdepth: Option<usize>,
    pub maxnodes: Option<usize>,
    pub movetime: Option<u128>,
    pub movestogo: Option<usize>,
}

impl Searcher {
    pub fn new(position: ataxx::Position, policy: policy::Fn, value: value::Fn) -> Searcher {
        Searcher {
            tree: Tree::new(position),
            params: Params::new(),
            policy,
            value,
        }
    }

    pub fn search(&mut self, limits: Limits, total_nodes: &mut u64) -> ataxx::Move {
        let maxdepth = limits.maxdepth.unwrap_or(usize::MAX);
        let maxnodes = limits.maxnodes.unwrap_or(usize::MAX);
        let movetime = limits.movetime.unwrap_or(u128::MAX);

        let start = time::Instant::now();

        let mut playouts = 0;

        let mut depth = 0;
        let mut seldepth = 0;
        let mut cumulative_depth = 0;

        loop {
            let mut new_depth = 0;
            self.playout(&mut new_depth);
            playouts += 1;

            cumulative_depth += new_depth;
            if new_depth > seldepth {
                seldepth = new_depth;
            }

            let avg_depth = cumulative_depth / playouts;
            if avg_depth > depth {
                depth = avg_depth;

                // Make a new info report.
                println!(
                    "info depth {} seldepth {} score cp {:.0} nodes {} nps {}",
                    depth,
                    seldepth,
                    0.0,
                    playouts,
                    self.nodes() * 1000 / start.elapsed().as_millis().max(1) as usize
                );
            }

            if playouts & 127 == 0 {
                if start.elapsed().as_millis() >= movetime
                    || depth >= maxdepth
                    || self.nodes() >= maxnodes
                {
                    break;
                }

                // Hard memory limit to prevent overuse.
                // TODO: Fix this by removing old nodes and stuff.
                if self.nodes() > 2_000_000_000 / mem::size_of::<Node>() {
                    break;
                }
            }
        }

        *total_nodes += self.nodes() as u64;

        println!(
            "info depth {} seldepth {} score cp {:.0} nodes {} nps {}",
            cumulative_depth / playouts,
            seldepth,
            100.0,
            playouts,
            self.nodes() * 1000 / start.elapsed().as_millis().max(1) as usize
        );

        // Verify the self.
        // debug_assert!(self.verify().is_ok());

        self.best_move()
    }
}

impl Searcher {
    pub fn playout(&mut self, depth: &mut usize) -> NodePtr {
        let mut position = self.root_position();
        let selected = self.select(&mut position, depth); // Select Node to be expanded
        let expanded = self.expand(selected, &mut position); // Expand the selected Node
        let simulate = self.simulate(&position); // Simulate the Node's result
        self.backpropagate(expanded, simulate); // Backpropagate the simulation

        expanded
    }

    fn select(&mut self, position: &mut ataxx::Position, depth: &mut usize) -> NodePtr {
        let policy = self.policy;
        let mut node_ptr: NodePtr = 0;

        loop {
            *depth += 1;

            let node = self.node_mut(node_ptr);

            if position.is_game_over() {
                break;
            }

            if !node.expanded() {
                // If the selected Node's Edges haven't been expanded, expand.
                node.expand(position, policy);
            }

            let node = self.node(node_ptr);

            // Select a new Edge from the current Node, and get the child Node.
            let edge = self.select_edge(node_ptr);
            let edge = node.edge(edge);

            if edge.ptr == -1 {
                // Selected Edge hasn't been expanded, so end selection for expansion.
                break;
            }

            *position = position.after_move::<true>(edge.mov);

            // Replace the Node pointer with the newly selected Node.
            node_ptr = edge.ptr;
        }

        node_ptr
    }

    //                    v-----------------------v exploitation
    //  node-q + policy * cpuct * sqrt(node-visits) / (1 + child-visits = 0) // not expanded
    // child-q + policy * cpuct * sqrt(node-visits) / (1 + child-visits)     // expanded
    // ^-----^ score / visits
    fn select_edge(&self, ptr: NodePtr) -> EdgePtr {
        let node = self.node(ptr);
        let parent = self.edge(node.parent_node, node.parent_edge);

        // Node exploitation factor (cpuct * sqrt(parent-playouts))
        let e = self.params.cpuct() * f64::sqrt(parent.visits.max(1) as f64);

        let mut best_ptr: EdgePtr = -1;
        let mut best_uct = 0.0;

        for (ptr, edge) in node.edges.iter().enumerate() {
            // If the edge hasn't been expanded yet, use the parent's q value.
            let q = if edge.ptr == -1 { parent.q() } else { edge.q() };

            let child_uct = q + edge.policy * e / (edge.visits as f64 + 1.0);

            // Check if we have a better UCT score for this edge.
            if child_uct > best_uct {
                best_ptr = ptr as EdgePtr;
                best_uct = child_uct;
            }
        }

        best_ptr
    }

    fn expand(&mut self, parent: NodePtr, position: &mut ataxx::Position) -> NodePtr {
        if position.is_game_over() {
            return parent;
        }

        // Select an Edge to expand from the current Node.
        let edge_ptr = self.select_edge(parent);

        let node = self.node(parent);
        let edge = node.edge(edge_ptr);

        *position = position.after_move::<true>(edge.mov);

        // Expand the Edge into a new Node.
        let new_node = Node::new(parent, edge_ptr);

        // Add the new Node to the Tree.
        let new_ptr = self.push_node(new_node);
        let edge = self.edge_mut(parent, edge_ptr);

        // Make the Edge point to the new Node.
        edge.ptr = new_ptr;

        new_ptr
    }

    fn simulate(&mut self, position: &ataxx::Position) -> f64 {
        if position.is_game_over() {
            let winner = position.winner();
            return if winner == ataxx::Piece::None {
                0.5 // Draw
            } else if winner == position.side_to_move {
                1.0 // Win
            } else {
                0.0 // Loss
            };
        };

        (self.value)(position)
    }

    fn backpropagate(&mut self, ptr: NodePtr, result: f64) {
        let mut node_ptr = ptr;
        let mut result = result;

        loop {
            let node = self.node(node_ptr);
            let parent_node = node.parent_node;
            let parent_edge = node.parent_edge;

            let edge = self.edge_mut(parent_node, parent_edge);

            edge.visits += 1;
            edge.scores += result;

            // Stop backpropagation if root has been reached.
            if node_ptr == 0 {
                break;
            }

            node_ptr = parent_node;
            result = 1.0 - result;
        }
    }
}

impl ops::Deref for Searcher {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl ops::DerefMut for Searcher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}
