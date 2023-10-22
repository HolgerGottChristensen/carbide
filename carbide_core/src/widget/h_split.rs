
use carbide_macro::{carbide_default_builder2};

use crate::CommonWidgetImpl;
use crate::cursor::MouseCursor;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent};
use crate::layout::Layout;
use crate::state::{IntoState, State};
use crate::widget::{CommonWidget, CrossAxisAlignment, SplitType, Widget, WidgetExt, WidgetId};

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout, MouseEvent, OtherEvent)]
pub struct HSplit<T> where T: State<T=f64> + Clone {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    // Leading - Trailing
    children: Vec<Box<dyn Widget>>,
    split: SplitType<T>,
    cross_axis_alignment: CrossAxisAlignment,
    dragging: bool,
    hovering: bool,
    draggable: bool,
}

impl HSplit<f64> {

    #[carbide_default_builder2]
    pub fn new(leading: Box<dyn Widget>, trailing: Box<dyn Widget>) -> Box<Self> {
        Self::new_internal(leading, trailing, SplitType::Percent(0.1), true)
    }

    fn new_internal<T: State<T=f64> + Clone>(
        leading: Box<dyn Widget>,
        trailing: Box<dyn Widget>,
        split: SplitType<T>,
        draggable: bool,
    ) -> Box<HSplit<T>> {
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

impl<T: State<T=f64> + Clone> HSplit<T> {
    pub fn relative_to_start<T2: IntoState<f64>>(mut self, width: T2) -> Box<HSplit<T2::Output>> {
        HSplit::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            SplitType::Start(width.into_state()),
            self.draggable,
        )
    }

    pub fn percent<T2: IntoState<f64>>(mut self, percent: T2) -> Box<HSplit<T2::Output>> {
        HSplit::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            SplitType::Percent(percent.into_state()),
            self.draggable,
        )
    }

    pub fn relative_to_end<T2: IntoState<f64>>(mut self, width: T2) -> Box<HSplit<T2::Output>> {
        HSplit::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            SplitType::End(width.into_state()),
            self.draggable,
        )
    }

    pub fn non_draggable(mut self) -> Box<Self> {
        HSplit::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            self.split,
            false,
        )
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Box<Self> {
        self.cross_axis_alignment = alignment;
        Box::new(self)
    }
}

impl<T: State<T=f64> + Clone> OtherEventHandler for HSplit<T> {
    fn handle_other_event(&mut self, _event: &WidgetEvent, env: &mut Environment) {
        if self.dragging || self.hovering {
            env.set_cursor(MouseCursor::ColResize);
        }
    }
}

impl<T: State<T=f64> + Clone> MouseEventHandler for HSplit<T> {
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

impl<T: State<T=f64> + Clone> Layout for HSplit<T> {
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

    fn position_children(&mut self, env: &mut Environment) {
        let position = self.position();
        let dimension = self.dimension();
        let alignment = self.cross_axis_alignment;

        let mut main_axis_offset = 0.0;

        self.foreach_child_mut(&mut |child| {
            let cross = match alignment {
                CrossAxisAlignment::Start => position.y,
                CrossAxisAlignment::Center => {
                    position.y + dimension.height / 2.0 - child.dimension().height / 2.0
                }
                CrossAxisAlignment::End => position.y + dimension.height - child.dimension().height,
            };

            child.set_position(Position::new(position.x + main_axis_offset, cross));
            main_axis_offset += child.dimension().width;
            child.position_children(env);
        });
    }
}

impl<T: State<T=f64> + Clone> CommonWidget for HSplit<T> {
    CommonWidgetImpl!(self, id: self.id, child: self.children, position: self.position, dimension: self.dimension);
}

impl<T: State<T=f64> + Clone> WidgetExt for HSplit<T> {}
