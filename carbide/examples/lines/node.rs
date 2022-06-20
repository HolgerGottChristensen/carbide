use std::slice::Iter;
use carbide_core::draw::Position;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Node {
    pub id: usize,
    pub position: Position,
    pub incoming_edges: Vec<usize>,
    pub outgoing_edges: Vec<usize>,
    pub hovered: bool,
}

impl Node {
    pub fn new(pos: Position) -> Node {
        Node {
            id: 0,
            position: pos,
            incoming_edges: vec![],
            outgoing_edges: vec![],
            hovered: false
        }
    }

    pub fn get_incoming_edges_iter(&self) -> Iter<usize> {
        self.incoming_edges.iter()
    }

    pub fn get_outgoing_edges_iter(&self) -> Iter<usize> {
        self.outgoing_edges.iter()
    }
}