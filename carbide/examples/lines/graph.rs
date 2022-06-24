use carbide_core::draw::Position;
use crate::edge::Edge;
use crate::editing_mode::EditingMode;
use crate::guide::Guide;
use crate::node::Node;

#[derive(Clone, Debug)]
pub struct Graph {
    pub offset: Position,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,

    pub guides: Vec<Guide>,
    pub editing_mode: EditingMode,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            offset: Default::default(),
            nodes: vec![],
            edges: vec![],
            guides: vec![],
            editing_mode: EditingMode::Normal
        }
    }

    pub fn get_node(&self, node_id: usize) -> &Node {
        &self.nodes[node_id as usize]
    }

    pub fn get_node_mut(&mut self, node_id: usize) -> &mut Node {
        &mut self.nodes[node_id as usize]
    }

    pub fn get_edge(&self, edge_id: usize) -> &Edge {
        &self.edges[edge_id as usize]
    }

    pub fn get_edge_mut(&mut self, edge_id: usize) -> &mut Edge {
        &mut self.edges[edge_id as usize]
    }

    pub fn add_node(&mut self, mut node: Node) -> usize {
        let id = self.nodes.len();
        node.id = id;
        self.nodes.push(node);
        id
    }

    pub fn add_edge(&mut self, from_id: usize, to_id: usize, mut edge: Edge) -> usize {
        let id = self.edges.len();
        edge.id = id;

        let from = self.get_node_mut(from_id);
        from.outgoing_edges.push(id);
        edge.from = from_id;

        let to = self.get_node_mut(to_id);
        to.incoming_edges.push(id);
        edge.to = to_id;

        self.edges.push(edge);

        id
    }

    pub fn get_connected_edges_iter(
        &self,
        node_id: usize,
    ) -> impl Iterator<Item = &Edge> + '_ {
        self
            .get_incoming_edges_iter(node_id)
            .chain(self.get_outgoing_edges_iter(node_id))
    }

    pub fn get_connected_neighbours_iter(
        &self,
        node_id: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        self
            .get_incoming_edges_iter(node_id)
            .map(|a| a.from)
            .chain(self.get_outgoing_edges_iter(node_id).map(|a| a.to))
    }

    pub fn get_incoming_edges_iter(
        &self,
        node_id: usize,
    ) -> impl Iterator<Item = &Edge> + '_ {
        let node = self.get_node(node_id);

        node
            .get_incoming_edges_iter()
            .map(move |edge_id: &usize| -> &Edge {
                let edge = self.get_edge(*edge_id);
                edge
            })
    }

    pub fn get_outgoing_edges_iter(
        &self,
        node_id: usize,
    ) -> impl Iterator<Item = &Edge> + '_ {
        let node = self.get_node(node_id);

        node
            .get_outgoing_edges_iter()
            .map(move |edge_id: &usize| -> &Edge {
                let edge = self.get_edge(*edge_id);
                edge
            })
    }

    pub fn split_edge_with_node(&mut self, edge_id: usize, pos: Position) {
        let new_id = self.add_node(Node::new(pos));
        let old_to = self.get_edge_mut(edge_id).to;

        self.get_node_mut(old_to).incoming_edges.retain(|a| *a != edge_id);

        self.get_edge_mut(edge_id).to = new_id;

        self.get_node_mut(new_id).incoming_edges.push(edge_id);

        self.add_edge(new_id, old_to, Edge::new());
    }
}