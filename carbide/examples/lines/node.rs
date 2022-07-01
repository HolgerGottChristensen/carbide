use std::slice::Iter;
use carbide_core::draw::Position;
use carbide_core::Scalar;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: usize,
    pub position: Position,
    pub incoming_edges: Vec<usize>,
    pub outgoing_edges: Vec<usize>,
    pub hovered: bool,
    pub height: Result<Scalar, String>,
}

impl Node {
    pub fn new(pos: Position) -> Node {
        Node {
            id: 0,
            position: pos,
            incoming_edges: vec![],
            outgoing_edges: vec![],
            hovered: false,
            height: Ok(210.0)
        }
    }

    pub fn get_incoming_edges_iter(&self) -> Iter<usize> {
        self.incoming_edges.iter()
    }

    pub fn get_outgoing_edges_iter(&self) -> Iter<usize> {
        self.outgoing_edges.iter()
    }
}