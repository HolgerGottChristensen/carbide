use crate::{CreateWallState, Edge, EditingMode, Graph, Line, Node, SelectedState};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{ModifierKey, MouseEvent, MouseEventHandler};
use carbide_core::state::{LocalState, ReadState, State, TState};
use carbide_core::widget::{CommonWidget, Widget, WidgetExt, WidgetId};
use carbide_core::CommonWidgetImpl;
use std::iter::once;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(MouseEvent)]
pub struct NodeEditor {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state]
    graph: TState<Graph>,
    #[state]
    selected_node: TState<Option<usize>>,
}

impl NodeEditor {
    pub fn new(graph: &TState<Graph>) -> Box<Self> {
        Box::new(Self {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            graph: graph.clone(),
            selected_node: LocalState::new(None),
        })
    }

    fn normal_mode_mouse_event(
        &mut self,
        event: &MouseEvent,
        consumed: &bool,
        env: &mut Environment,
    ) {
        match event {
            MouseEvent::Press(_, b, _) => {
                let b = *b - self.position;
                let mut closest_id = 0;
                let mut closest_distance = self.graph.value().get_node(0).position.dist(&b);

                for node in &self.graph.value().nodes {
                    let dist = node.position.dist(&b);
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
            MouseEvent::Move {
                from,
                to,
                delta_xy,
                modifiers,
            } => {
                let to = *to - self.position;
                if *self.selected_node.value() == None {
                    for node in &mut self.graph.value_mut().nodes {
                        if node.position.dist(&to) < 10.0 {
                            node.hovered = true;
                        } else {
                            node.hovered = false;
                        }
                    }
                }

                if let Some(id) = *self.selected_node.value() {
                    let mut new_position = self.graph.value_mut().guides_and_position(to, id);
                    self.graph.value_mut().get_node_mut(id).position = new_position;
                }
            }
            MouseEvent::NClick(_, position, _, number_of_clicks) => {
                if *number_of_clicks != 2 {
                    return;
                }

                let position = *position - self.position;
                if *self.selected_node.value() == None {
                    let close_line = self.graph.value().edge_in_range(position);

                    if let Some(edge_id) = close_line {
                        let new_pos = {
                            let graph = self.graph.value();
                            let edge = graph.get_edge(edge_id);

                            let line = Line::new(
                                graph.get_node(edge.from).position,
                                graph.get_node(edge.to).position,
                            );

                            line.closest_point_on_line_infinite(position)
                        };

                        self.graph
                            .value_mut()
                            .split_edge_with_node(edge_id, new_pos);
                    }
                }
            }
            MouseEvent::Scroll {
                x,
                y,
                mouse_position,
                modifiers,
            } => {
                //self.graph.value_mut().offset -= Position::new(*x, *y);
            }
            MouseEvent::Drag { .. } => {}
        }
    }

    fn create_wall_p1_mouse_event(
        &mut self,
        event: &MouseEvent,
        consumed: &bool,
        env: &mut Environment,
    ) {
        match event {
            MouseEvent::Press(_, _, _) => {}
            MouseEvent::Release(_, to, _) => {
                let to = *to - self.position;
                let close_node = self.graph.value().node_in_range(to);
                let close_edge = self.graph.value().edge_in_range(to);
                if let Some(close_node_id) = close_node {
                    let pos = self.graph.value().get_node(close_node_id).position;
                    self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                        mouse_position: pos,
                        state: CreateWallState::ExistingNode,
                        first_node_id: close_node_id,
                    }
                } else if let Some(close_edge_id) = close_edge {
                    let new_pos = {
                        let graph = self.graph.value();
                        let edge = graph.get_edge(close_edge_id);

                        let line = Line::new(
                            graph.get_node(edge.from).position,
                            graph.get_node(edge.to).position,
                        );

                        line.closest_point_on_line_infinite(to)
                    };

                    let id = self
                        .graph
                        .value_mut()
                        .split_edge_with_node(close_edge_id, new_pos);

                    self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                        mouse_position: new_pos,
                        first_node_id: id,
                        state: CreateWallState::SplitEdge,
                    }
                }
            }
            MouseEvent::Click(_, _, _) => {}
            MouseEvent::Move { to, .. } => {
                let to = *to - self.position;
                let close_node = self.graph.value().node_in_range(to);
                let close_edge = self.graph.value().edge_in_range(to);
                if let Some(close_node_id) = close_node {
                    let pos = self.graph.value().get_node(close_node_id).position;
                    self.graph.value_mut().editing_mode = EditingMode::CreateWallP1 {
                        mouse_position: pos,
                        state: CreateWallState::ExistingNode,
                    }
                } else if let Some(close_edge_id) = close_edge {
                    let new_pos = {
                        let graph = self.graph.value();
                        let edge = graph.get_edge(close_edge_id);

                        let line = Line::new(
                            graph.get_node(edge.from).position,
                            graph.get_node(edge.to).position,
                        );

                        line.closest_point_on_line_infinite(to)
                    };

                    self.graph.value_mut().editing_mode = EditingMode::CreateWallP1 {
                        mouse_position: new_pos,
                        state: CreateWallState::SplitEdge,
                    }
                } else {
                    self.graph.value_mut().editing_mode = EditingMode::CreateWallP1 {
                        mouse_position: to,
                        state: CreateWallState::Invalid,
                    }
                }
            }
            MouseEvent::NClick(_, _, _, _) => {}
            MouseEvent::Scroll { .. } => {}
            MouseEvent::Drag { .. } => {}
        }
    }

    fn create_wall_p2_mouse_event(
        &mut self,
        first_id: usize,
        event: &MouseEvent,
        consumed: &bool,
        env: &mut Environment,
    ) {
        match event {
            MouseEvent::Release(_, to, modifier) => {
                let to = *to - self.position;
                let close_node = self.graph.value().node_in_range(to);
                let close_edge = self.graph.value().edge_in_range(to);
                if let Some(close_node_id) = close_node {
                    let pos = self.graph.value().get_node(close_node_id).position;

                    let valid = !self
                        .graph
                        .value()
                        .get_connected_neighbours_iter(first_id)
                        .chain(once(first_id))
                        .any(|a| a == close_node_id);

                    if valid {
                        self.graph
                            .value_mut()
                            .add_edge(first_id, close_node_id, Edge::new());
                        self.graph.value_mut().editing_mode = EditingMode::Editing;
                    } else {
                        self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                            mouse_position: pos,
                            first_node_id: first_id,
                            state: CreateWallState::Invalid,
                        }
                    }
                } else if let Some(close_edge_id) = close_edge {
                    let new_pos = {
                        let graph = self.graph.value();
                        let edge = graph.get_edge(close_edge_id);

                        let line = Line::new(
                            graph.get_node(edge.from).position,
                            graph.get_node(edge.to).position,
                        );

                        line.closest_point_on_line_infinite(to)
                    };

                    let valid = !self
                        .graph
                        .value()
                        .get_connected_edges_iter(first_id)
                        .any(|a| a.id == close_edge_id);

                    if valid {
                        let id = self
                            .graph
                            .value_mut()
                            .split_edge_with_node(close_edge_id, new_pos);

                        self.graph.value_mut().add_edge(first_id, id, Edge::new());

                        self.graph.value_mut().editing_mode = EditingMode::Editing;
                    } else {
                        self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                            mouse_position: new_pos,
                            first_node_id: first_id,
                            state: CreateWallState::Invalid,
                        }
                    }
                } else {
                    let mut new_position =
                        self.graph.value_mut().guides_and_position(to, usize::MAX);

                    let new = self.graph.value_mut().add_node(Node::new(new_position));

                    if modifier.contains(ModifierKey::SHIFT) {
                        self.graph.value_mut().add_edge(first_id, new, Edge::new());
                        self.graph.value_mut().editing_mode = EditingMode::Editing;
                    } else {
                        self.graph.value_mut().add_edge(first_id, new, Edge::new());
                        self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                            mouse_position: new_position,
                            first_node_id: new,
                            state: CreateWallState::Floating,
                        }
                    }
                }
            }
            MouseEvent::Move { to, .. } => {
                let to = *to - self.position;
                let close_node = self.graph.value().node_in_range(to);
                let close_edge = self.graph.value().edge_in_range(to);

                self.graph.value_mut().guides.clear();

                if let Some(close_node_id) = close_node {
                    let pos = self.graph.value().get_node(close_node_id).position;

                    let valid = !self
                        .graph
                        .value()
                        .get_connected_neighbours_iter(first_id)
                        .chain(once(first_id))
                        .any(|a| a == close_node_id);

                    if valid {
                        self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                            mouse_position: pos,
                            first_node_id: first_id,
                            state: CreateWallState::ExistingNode,
                        }
                    } else {
                        self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                            mouse_position: pos,
                            first_node_id: first_id,
                            state: CreateWallState::Invalid,
                        }
                    }
                } else if let Some(close_edge_id) = close_edge {
                    let new_pos = {
                        let graph = self.graph.value();
                        let edge = graph.get_edge(close_edge_id);

                        let line = Line::new(
                            graph.get_node(edge.from).position,
                            graph.get_node(edge.to).position,
                        );

                        line.closest_point_on_line_infinite(to)
                    };

                    let valid = !self
                        .graph
                        .value()
                        .get_connected_edges_iter(first_id)
                        .any(|a| a.id == close_edge_id);

                    if valid {
                        self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                            mouse_position: new_pos,
                            first_node_id: first_id,
                            state: CreateWallState::SplitEdge,
                        }
                    } else {
                        self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                            mouse_position: new_pos,
                            first_node_id: first_id,
                            state: CreateWallState::Invalid,
                        }
                    }
                } else {
                    let mut new_position =
                        self.graph.value_mut().guides_and_position(to, usize::MAX);

                    self.graph.value_mut().editing_mode = EditingMode::CreateWallP2 {
                        mouse_position: new_position,
                        first_node_id: first_id,
                        state: CreateWallState::Floating,
                    };
                }
            }
            _ => (),
        }
    }

    fn selection_mode_mouse_event(
        &mut self,
        event: &MouseEvent,
        consumed: &bool,
        env: &mut Environment,
        selected: SelectedState,
    ) {
        match event {
            MouseEvent::Press(_, _, _) => {}
            MouseEvent::Release(_, _, _) => {}
            MouseEvent::Click(_, to, _) => {
                if !self.is_inside(*to) {
                    return;
                }
                let to = *to - self.position;

                let close_node = self.graph.value().node_in_range(to);
                let close_edge = self.graph.value().edge_in_range(to);

                let closest_selection = if let Some(node_id) = close_node {
                    SelectedState::Node(node_id)
                } else if let Some(edge_id) = close_edge {
                    SelectedState::Edge(edge_id)
                } else {
                    SelectedState::None
                };

                self.graph.value_mut().editing_mode = EditingMode::Selection {
                    selected: closest_selection,
                    hovered: closest_selection,
                }
            }
            MouseEvent::Move { to, .. } => {
                let to = *to - self.position;

                let close_node = self.graph.value().node_in_range(to);
                let close_edge = self.graph.value().edge_in_range(to);

                let closest_selection = if let Some(node_id) = close_node {
                    SelectedState::Node(node_id)
                } else if let Some(edge_id) = close_edge {
                    SelectedState::Edge(edge_id)
                } else {
                    SelectedState::None
                };

                self.graph.value_mut().editing_mode = EditingMode::Selection {
                    selected,
                    hovered: closest_selection,
                }
            }
            MouseEvent::NClick(_, _, _, _) => {}
            MouseEvent::Scroll { .. } => {}
            MouseEvent::Drag { .. } => {}
        }
    }
}

impl MouseEventHandler for NodeEditor {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        let mode = self.graph.value().editing_mode.clone();
        match mode {
            EditingMode::Editing => {
                self.normal_mode_mouse_event(event, consumed, env);
            }
            EditingMode::CreateWallP1 { .. } => {
                self.create_wall_p1_mouse_event(event, consumed, env);
            }
            EditingMode::CreateWallP2 { first_node_id, .. } => {
                self.create_wall_p2_mouse_event(first_node_id, event, consumed, env);
            }
            EditingMode::Selection { selected, .. } => {
                self.selection_mode_mouse_event(event, consumed, env, selected);
            }
        }
    }
}

CommonWidgetImpl!(NodeEditor, self, id: self.id, position: self.position, dimension: self.dimension);

impl WidgetExt for NodeEditor {}
