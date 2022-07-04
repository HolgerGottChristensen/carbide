use crate::cursor::MouseCursor;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent};
use crate::layout::Layout;
use crate::state::{F64State, LocalState, ReadState, State};
use crate::widget::{CommonWidget, CrossAxisAlignment, SplitType, Widget, WidgetExt, WidgetId};
use crate::CommonWidgetImpl;
use crate::Widget;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout, MouseEvent, OtherEvent)]
pub struct HSplit {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    // Leading - Trailing
    children: Vec<Box<dyn Widget>>,
    split: SplitType,
    cross_axis_alignment: CrossAxisAlignment,
    dragging: bool,
    hovering: bool,
    draggable: bool,
}

impl HSplit {
    pub fn new(leading: Box<dyn Widget>, trailing: Box<dyn Widget>) -> Box<Self> {
        let split = LocalState::new(0.1);
        Self::new_internal(leading, trailing, SplitType::Percent(split), true)
    }

    pub fn relative_to_start(mut self, width: impl Into<F64State>) -> Box<Self> {
        Self::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            SplitType::Start(width.into()),
            self.draggable,
        )
    }

    pub fn percent(mut self, percent: impl Into<F64State>) -> Box<Self> {
        Self::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            SplitType::Percent(percent.into()),
            self.draggable,
        )
    }

    pub fn non_draggable(mut self) -> Box<Self> {
        Self::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            self.split,
            false,
        )
    }

    pub fn relative_to_end(mut self, width: impl Into<F64State>) -> Box<Self> {
        Self::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            SplitType::End(width.into()),
            self.draggable,
        )
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Box<Self> {
        self.cross_axis_alignment = alignment;
        Box::new(self)
    }

    fn new_internal(
        leading: Box<dyn Widget>,
        trailing: Box<dyn Widget>,
        split: SplitType,
        draggable: bool,
    ) -> Box<Self> {
        Box::new(HSplit {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            children: vec![leading, trailing],
            split,
            cross_axis_alignment: CrossAxisAlignment::Center,
            dragging: false,
            hovering: false,
            draggable,
        })
    }
}

impl OtherEventHandler for HSplit {
    fn handle_other_event(&mut self, _event: &WidgetEvent, env: &mut Environment) {
        if self.dragging || self.hovering {
            env.set_cursor(MouseCursor::ColResize);
        }
    }
}

impl MouseEventHandler for HSplit {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {
        if !self.draggable {
            return;
        }

        let press_margin = 5.0;

        match event {
            MouseEvent::Press(_, position, _) => {
                let relative_to_position = *position - self.position;

                let split = self.children[0].dimension();

                if relative_to_position.x > split.width - press_margin
                    && relative_to_position.x < split.width + press_margin
                    && relative_to_position.y <= split.height
                    && relative_to_position.y > 0.0
                {
                    self.dragging = true;
                }
            }
            MouseEvent::Release(_, _, _) => {
                self.dragging = false;
            }
            MouseEvent::Move { to, .. } => {
                let relative_to_position = *to - self.position;
                let split = self.children[0].dimension();

                if relative_to_position.x > split.width - press_margin
                    && relative_to_position.x < split.width + press_margin
                    && relative_to_position.y <= split.height
                    && relative_to_position.y > 0.0
                {
                    self.hovering = true;
                } else {
                    self.hovering = false;
                }

                if !self.dragging {
                    return;
                }

                let width = self.width();

                match &mut self.split {
                    SplitType::Start(offset) => {
                        let new_offset = relative_to_position.x;
                        offset.set_value(new_offset.max(0.0).min(width));
                    }
                    SplitType::Percent(percent) => {
                        let p = relative_to_position.x / self.dimension.width;
                        percent.set_value(p.max(0.0).min(1.0));
                    }
                    SplitType::End(offset) => {
                        let new_offset = width - relative_to_position.x;
                        offset.set_value(new_offset.max(0.0).min(width));
                    }
                }
            }
            _ => (),
        }
    }
}

impl Layout for HSplit {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let (requested_leading_width, requested_trailing_width) = match &self.split {
            SplitType::Start(offset) => (*offset.value(), requested_size.width - *offset.value()),
            SplitType::Percent(percent) => {
                let leading = requested_size.width * *percent.value();
                let trailing = requested_size.width * (1.0 - *percent.value());
                (leading, trailing)
            }
            SplitType::End(offset) => (requested_size.width - *offset.value(), *offset.value()),
        };

        let leading_size = Dimension::new(requested_leading_width, requested_size.height);
        let mut leading = self.children[0].calculate_size(leading_size, env);

        let trailing_size = Dimension::new(requested_trailing_width, requested_size.height);
        let mut trailing = self.children[1].calculate_size(trailing_size, env);

        if leading.width > requested_leading_width {
            let trailing_size =
                Dimension::new(requested_size.width - leading.width, requested_size.height);
            trailing = self.children[1].calculate_size(trailing_size, env);
        } else if trailing.width > requested_trailing_width {
            let leading_size =
                Dimension::new(requested_size.width - trailing.width, requested_size.height);
            leading = self.children[0].calculate_size(leading_size, env);
        }

        self.set_dimension(Dimension::new(
            requested_size.width,
            leading.height.max(trailing.height),
        ));
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
                CrossAxisAlignment::End => position.y + dimension.height - child.dimension().height,
            };

            child.set_position(Position::new(position.x + main_axis_offset, cross));
            main_axis_offset += child.dimension().width;
            child.position_children();
        }
    }
}

CommonWidgetImpl!(HSplit, self, id: self.id, children: self.children, position: self.position, dimension: self.dimension);

impl WidgetExt for HSplit {}
