use crate::CommonWidgetImpl;
use crate::cursor::MouseCursor;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent};
use crate::layout::Layout;
use crate::state::{F64State, LocalState, State};
use crate::widget::{CommonWidget, CrossAxisAlignment, Id, Widget, WidgetExt};
use crate::Widget;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout, MouseEvent, OtherEvent)]
pub struct VSplit {
    id: Id,
    position: Position,
    dimension: Dimension,
    // Top - Bottom
    children: Vec<Box<dyn Widget>>,
    split: F64State,
    cross_axis_alignment: CrossAxisAlignment,
    dragging: bool,
    hovering: bool,
}

impl VSplit {
    pub fn new(leading: Box<dyn Widget>, trailing: Box<dyn Widget>) -> Box<Self> {
        let split = LocalState::new(0.1);
        Self::new_internal(leading, trailing, split)
    }

    pub fn percent(mut self, percent: impl Into<F64State>) -> Box<Self> {
        Self::new_internal(self.children.remove(0), self.children.remove(0), percent.into())
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Box<Self> {
        self.cross_axis_alignment = alignment;
        Box::new(self)
    }

    fn new_internal(leading: Box<dyn Widget>, trailing: Box<dyn Widget>, split: impl Into<F64State>) -> Box<Self> {
        Box::new(VSplit {
            id: Id::new_v4(),
            position: Default::default(),
            dimension: Default::default(),
            children: vec![leading, trailing],
            split: split.into(),
            cross_axis_alignment: CrossAxisAlignment::Center,
            dragging: false,
            hovering: false,
        })
    }
}

impl OtherEventHandler for VSplit {
    fn handle_other_event(&mut self, _event: &WidgetEvent, env: &mut Environment) {
        if self.dragging || self.hovering {
            env.set_cursor(MouseCursor::RowResize);
        }
    }
}

impl MouseEventHandler for VSplit {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {
        let press_margin = 5.0;

        match event {
            MouseEvent::Press(_, position, _) => {
                let relative_to_position = *position - self.position;

                let split = self.children[0].dimension();

                if relative_to_position.y > split.height - press_margin &&
                    relative_to_position.y < split.height + press_margin {
                    self.dragging = true;
                }
            }
            MouseEvent::Release(_, _, _) => {
                self.dragging = false;
            }
            MouseEvent::Move { to, .. } => {
                let relative_to_position = *to - self.position;

                let split = self.children[0].dimension();
                if relative_to_position.y > split.height - press_margin &&
                    relative_to_position.y < split.height + press_margin {
                    self.hovering = true;
                } else {
                    self.hovering = false;
                }

                if !self.dragging { return; }

                let percent = relative_to_position.y / self.dimension.height;

                self.split.set_value(percent.max(0.0).min(1.0))
            }
            _ => ()
        }
    }
}

impl Layout for VSplit {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let requested_top_height = requested_size.height * *self.split.value();
        let requested_bottom_height = requested_size.height * (1.0 - *self.split.value());

        let top_size = Dimension::new(requested_size.width, requested_top_height);
        let mut top = self.children[0].calculate_size(top_size, env);

        let bottom_size = Dimension::new(requested_size.width, requested_bottom_height);
        let mut bottom = self.children[1].calculate_size(bottom_size, env);

        if top.height > requested_top_height {
            let bottom_size = Dimension::new(requested_size.width, requested_size.height - top.height);
            bottom = self.children[1].calculate_size(bottom_size, env);
        } else if bottom.height > requested_bottom_height {
            let top_size = Dimension::new(requested_size.width, requested_size.height - bottom.height);
            top = self.children[0].calculate_size(top_size, env);
        }

        self.set_dimension(Dimension::new(top.width.max(bottom.width), requested_size.height));
        self.dimension
    }

    fn position_children(&mut self) {
        let position = self.position();
        let dimension = self.dimension();
        let alignment = self.cross_axis_alignment;

        let mut main_axis_offset = 0.0;

        for mut child in self.children_mut() {
            let cross = match alignment {
                CrossAxisAlignment::Start => position.x,
                CrossAxisAlignment::Center => {
                    position.x + dimension.width / 2.0 - child.dimension().width / 2.0
                }
                CrossAxisAlignment::End => {
                    position.x + dimension.width - child.dimension().width
                }
            };

            child.set_position(Position::new(cross, position.y + main_axis_offset));
            main_axis_offset += child.dimension().height;
            child.position_children();
        }
    }
}

CommonWidgetImpl!(VSplit, self, id: self.id, children: self.children, position: self.position, dimension: self.dimension);

impl WidgetExt for VSplit {}