use std::ops::Deref;
use carbide::Widget;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{MouseEvent, MouseEventHandler};
use carbide_core::prelude::{State, WidgetId};
use carbide_core::state::{LocalState, ReadState, TState};
use carbide_core::widget::WidgetExt;
use crate::{Edge, Graph, Line, Node};
use crate::guide::Guide;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(MouseEvent)]
pub struct NodeEditor {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] graph: TState<Graph>,
    #[state] selected_node: TState<Option<usize>>,
}

impl NodeEditor {
    pub fn new(graph: TState<Graph>) -> Box<Self> {
        Box::new(
            Self {
                id: WidgetId::new(),
                position: Default::default(),
                dimension: Default::default(),
                graph,
                selected_node: LocalState::new(None),
            }
        )
    }
}

impl MouseEventHandler for NodeEditor {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        match event {
            MouseEvent::Press(a, b, c) => {
                let mut closest_id = 0;
                let mut closest_distance = self.graph.value().get_node(0).position.dist(b);

                for node in &self.graph.value().nodes {
                    let dist = node.position.dist(b);
                    if dist < closest_distance {
                        closest_distance = dist;
                        closest_id = node.id;
                    }
                }

                if closest_distance < 10.0 {
                    self.selected_node.set_value(Some(closest_id));
                } else {
                    self.selected_node.set_value(None);
                }
            }
            MouseEvent::Release(_, _, _) => {
                self.selected_node.set_value(None);
                self.graph.value_mut().guides.clear();
            }
            MouseEvent::Click(_, _, _) => {}
            MouseEvent::Move { from, to, delta_xy, modifiers } => {

                if *self.selected_node.value() == None {
                    for node in &mut self.graph.value_mut().nodes {
                        if node.position.dist(to) < 10.0 {
                            node.hovered = true;
                        } else {
                            node.hovered = false;
                        }
                    }
                }

                if let Some(id) = *self.selected_node.value() {
                    let mut new_position = *to;

                    let graph = self.graph.value();

                    let mut guides = vec![];




                    for edge_id in 0..graph.edges.len() {
                        let edge = graph.get_edge(edge_id);
                        if edge.to == id || edge.from == id {
                            continue;
                        }

                        let line = Line::new(
                            graph.get_node(edge.from).position,
                            graph.get_node(edge.to).position,
                        );

                        if line.dist_inf_line_to_point(*to) < 5.0 {
                            new_position = line.closest_point_on_line_infinite(*to);
                            guides.push(Guide::Directional(line));
                        }
                    }

                    //for neighbour_id in graph.get_connected_neighbours_iter(id) {
                    for neighbour_id in 0..graph.nodes.len() {
                        if neighbour_id == id {continue}

                        let node = graph.get_node(neighbour_id);

                        if (to.x() - node.position.x()).abs() < 5.0 {
                            guides.push(Guide::Vertical(node.position.x()));
                            new_position = Position::new(node.position.x(), new_position.y());
                        }

                        if (to.y() - node.position.y()).abs() < 5.0 {
                            guides.push(Guide::Horizontal(node.position.y()));
                            new_position = Position::new(new_position.x(), node.position.y());
                        }
                    }

                    drop(graph);

                    self.graph.value_mut().get_node_mut(id).position = new_position;

                    self.graph.value_mut().guides = guides;

                }
            }
            MouseEvent::NClick(_, position, _, number_of_clicks) => {
                if *number_of_clicks != 2 {return;}
                if *self.selected_node.value() == None {
                    let graph = self.graph.value();
                    let mut number_of_close_lines = 0;
                    let mut close_line_id = 0;

                    for edge_id in 0..graph.edges.len() {
                        let edge = graph.get_edge(edge_id);

                        let line = Line::new(
                            graph.get_node(edge.from).position,
                            graph.get_node(edge.to).position,
                        );

                        if let Some(_) = line.closest_point_on_line(*position) {
                            if line.dist_inf_line_to_point(*position) < 5.0 {
                                number_of_close_lines += 1;
                                close_line_id = edge_id;
                            }
                        }
                    }

                    drop(graph);

                    if number_of_close_lines == 1 {
                        let new_pos = {
                            let graph = self.graph.value();
                            let edge = graph.get_edge(close_line_id);

                            let line = Line::new(
                                graph.get_node(edge.from).position,
                                graph.get_node(edge.to).position,
                            );

                            line.closest_point_on_line_infinite(*position)
                        };

                        let new_id = self.graph.value_mut().add_node(Node::new(new_pos));
                        let old_to = self.graph.value_mut().get_edge_mut(close_line_id).to;

                        self.graph.value_mut().get_node_mut(old_to).incoming_edges.retain(|a| *a != close_line_id);

                        self.graph.value_mut().get_edge_mut(close_line_id).to = new_id;

                        self.graph.value_mut().get_node_mut(new_id).incoming_edges.push(close_line_id);

                        self.graph.value_mut().add_edge(new_id, old_to, Edge::new());
                    }
                }
            }
            MouseEvent::Scroll { x, y, mouse_position, modifiers } => {
                //self.graph.value_mut().offset -= Position::new(*x, *y);
            }
            MouseEvent::Drag { .. } => {}
        }
    }
}

CommonWidgetImpl!(NodeEditor, self, id: self.id, position: self.position, dimension: self.dimension);

impl WidgetExt for NodeEditor {}