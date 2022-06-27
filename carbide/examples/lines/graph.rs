use carbide_core::draw::Position;
use crate::edge::Edge;
use crate::editing_mode::EditingMode;
use crate::guide::Guide;
use crate::Line;
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
            editing_mode: EditingMode::Editing
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

    pub fn split_edge_with_node(&mut self, edge_id: usize, pos: Position) -> usize {
        let new_id = self.add_node(Node::new(pos));
        let old_to = self.get_edge_mut(edge_id).to;

        self.get_node_mut(old_to).incoming_edges.retain(|a| *a != edge_id);

        self.get_edge_mut(edge_id).to = new_id;

        self.get_node_mut(new_id).incoming_edges.push(edge_id);

        self.add_edge(new_id, old_to, Edge::new());

        new_id
    }

    pub fn node_in_range(&self, position: Position) -> Option<usize> {
        let mut number_of_close_nodes = 0;
        let mut close_node_id = 0;

        for node_id in 0..self.nodes.len() {
            let node = self.get_node(node_id);

            if node.position.dist(&position) < 5.0 {
                number_of_close_nodes += 1;
                close_node_id = node_id;
            }
        }

        if number_of_close_nodes != 1 {
            None
        } else {
            Some(close_node_id)
        }
    }

    pub fn edge_in_range(&self, position: Position) -> Option<usize> {
        let mut number_of_close_lines = 0;
        let mut close_line_id = 0;

        for edge_id in 0..self.edges.len() {
            let edge = self.get_edge(edge_id);

            let line = Line::new(
                self.get_node(edge.from).position,
                self.get_node(edge.to).position,
            );

            if let Some(_) = line.closest_point_on_line(position) {
                if line.dist_inf_line_to_point(position) < 5.0 {
                    number_of_close_lines += 1;
                    close_line_id = edge_id;
                }
            }
        }

        if number_of_close_lines != 1 {
            None
        } else {
            Some(close_line_id)
        }
    }

    pub fn guides_and_position(&mut self, position: Position, ignore_node_id: usize) -> Position {
        let mut new_position = position;
        let mut guides = vec![];

        for edge_id in 0..self.edges.len() {
            let edge = self.get_edge(edge_id);
            if edge.to == ignore_node_id || edge.from == ignore_node_id {
                continue;
            }

            let line = Line::new(
                self.get_node(edge.from).position,
                self.get_node(edge.to).position,
            );

            if line.dist_inf_line_to_point(position) < 5.0 {
                new_position = line.closest_point_on_line_infinite(position);
                guides.push(Guide::Directional(line));
            }
        }

        for neighbour_id in 0..self.nodes.len() {
            let node = self.get_node(neighbour_id);
            if neighbour_id == ignore_node_id {continue}

            if (position.x() - node.position.x()).abs() < 5.0 {
                guides.push(Guide::Vertical(node.position.x()));
                new_position = Position::new(node.position.x(), new_position.y());
            }

            if (position.y() - node.position.y()).abs() < 5.0 {
                guides.push(Guide::Horizontal(node.position.y()));
                new_position = Position::new(new_position.x(), node.position.y());
            }
        }

        self.guides = guides;

        new_position
    }
}