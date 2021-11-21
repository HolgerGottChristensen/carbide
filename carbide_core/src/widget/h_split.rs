use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{MouseEvent, MouseEventHandler};
use crate::layout::Layout;
use crate::state::{F64State, LocalState, State};
use crate::widget::{CommonWidget, CrossAxisAlignment, Id, Widget, WidgetExt};
use crate::Widget;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout, MouseEvent)]
pub struct HSplit {
    id: Id,
    position: Position,
    dimension: Dimension,
    // Leading - Trailing
    children: Vec<Box<dyn Widget>>,
    split: F64State,
    cross_axis_alignment: CrossAxisAlignment,
    dragging: bool,
}

impl HSplit {
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
        Box::new(HSplit {
            id: Id::new_v4(),
            position: Default::default(),
            dimension: Default::default(),
            children: vec![leading, trailing],
            split: split.into(),
            cross_axis_alignment: CrossAxisAlignment::Center,
            dragging: false,
        })
    }
}

impl MouseEventHandler for HSplit {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {
        match event {
            MouseEvent::Press(_, position, _) => {
                let relative_to_position = *position - self.position;
                let press_margin = 5.0;

                let split = self.children[0].dimension();

                if relative_to_position.x > split.width - press_margin &&
                    relative_to_position.x < split.width + press_margin {
                    self.dragging = true;
                }
            }
            MouseEvent::Release(_, _, _) => {
                self.dragging = false;
            }
            MouseEvent::Move { to, .. } => {
                if !self.is_inside(*to) || !self.dragging { return; }

                let relative_to_position = *to - self.position;
                let percent = relative_to_position.x / self.dimension.width;

                self.split.set_value(percent.max(0.0).min(1.0))
            }
            _ => ()
        }
    }
}

impl Layout for HSplit {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let requested_leading_width = requested_size.width * *self.split.value();
        let requested_trailing_width = requested_size.width * (1.0 - *self.split.value());

        let leading_size = Dimension::new(requested_leading_width, requested_size.height);
        let mut leading = self.children[0].calculate_size(leading_size, env);

        let trailing_size = Dimension::new(requested_trailing_width, requested_size.height);
        let mut trailing = self.children[1].calculate_size(trailing_size, env);

        if leading.width > requested_leading_width {
            let trailing_size = Dimension::new(requested_size.width - leading.width, requested_size.height);
            trailing = self.children[1].calculate_size(trailing_size, env);
        } else if trailing.width > requested_trailing_width {
            let leading_size = Dimension::new(requested_size.width - trailing.width, requested_size.height);
            leading = self.children[0].calculate_size(leading_size, env);
        }

        self.set_dimension(Dimension::new(requested_size.width, leading.height.max(trailing.height)));
        self.dimension
    }

    fn position_children(&mut self) {
        let position = self.position();
        let dimension = self.dimension();
        let alignment = self.cross_axis_alignment;

        let mut main_axis_offset = 0.0;

        for mut child in self.children_mut() {
            let cross = match alignment {
                CrossAxisAlignment::Start => position.y,
                CrossAxisAlignment::Center => {
                    position.y + dimension.height / 2.0 - child.dimension().height / 2.0
                }
                CrossAxisAlignment::End => {
                    position.y + dimension.height - child.dimension().height
                }
            };

            child.set_position(Position::new(position.x + main_axis_offset, cross));
            main_axis_offset += child.dimension().width;
            child.position_children();
        }
    }
}

CommonWidgetImpl!(HSplit, self, id: self.id, children: self.children, position: self.position, dimension: self.dimension);

impl WidgetExt for HSplit {}