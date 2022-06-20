use carbide::Widget;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{MouseEvent, MouseEventHandler};
use carbide_core::prelude::{State, WidgetId};
use carbide_core::state::{LocalState, ReadState, TState};
use carbide_core::widget::WidgetExt;
use crate::Graph;

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
                    self.graph.value_mut().get_node_mut(id).position = *to;
                }
            }
            MouseEvent::NClick(_, _, _, _) => {}
            MouseEvent::Scroll { .. } => {}
            MouseEvent::Drag { .. } => {}
        }
    }
}

CommonWidgetImpl!(NodeEditor, self, id: self.id, position: self.position, dimension: self.dimension);

impl WidgetExt for NodeEditor {}