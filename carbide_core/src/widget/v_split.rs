
use carbide_macro::{carbide_default_builder2};

use crate::CommonWidgetImpl;
use crate::cursor::MouseCursor;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent};
use crate::layout::Layout;
use crate::state::{IntoState, State};
use crate::widget::{CommonWidget, CrossAxisAlignment, SplitType, AnyWidget, WidgetExt, WidgetId, Widget};

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout, MouseEvent, OtherEvent)]
pub struct VSplit<T> where T: State<T=f64> + Clone {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    // Top - Bottom
    children: Vec<Box<dyn AnyWidget>>,
    split: SplitType<T>,
    cross_axis_alignment: CrossAxisAlignment,
    dragging: bool,
    hovering: bool,
}

impl VSplit<f64> {
    #[carbide_default_builder2]
    pub fn new(leading: Box<dyn AnyWidget>, trailing: Box<dyn AnyWidget>) -> Box<Self> {
        Self::new_internal(leading, trailing, SplitType::Percent(0.1))
    }

    fn new_internal<T: State<T=f64> + Clone>(
        leading: Box<dyn AnyWidget>,
        trailing: Box<dyn AnyWidget>,
        split: SplitType<T>,
    ) -> Box<VSplit<T>> {
        Box::new(VSplit {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            children: vec![leading, trailing],
            split,
            cross_axis_alignment: CrossAxisAlignment::Center,
            dragging: false,
            hovering: false,
        })
    }
}

impl<T: State<T=f64> + Clone> VSplit<T> {
    pub fn relative_to_start<T2: IntoState<f64>>(mut self, width: T2) -> Box<VSplit<T2::Output>> {
        VSplit::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            SplitType::Start(width.into_state()),
        )
    }

    pub fn percent<T2: IntoState<f64>>(mut self, percent: T2) -> Box<VSplit<T2::Output>> {
        VSplit::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            SplitType::Percent(percent.into_state()),
        )
    }

    pub fn relative_to_end<T2: IntoState<f64>>(mut self, width: T2) -> Box<VSplit<T2::Output>> {
        VSplit::new_internal(
            self.children.remove(0),
            self.children.remove(0),
            SplitType::End(width.into_state()),
        )
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Box<Self> {
        self.cross_axis_alignment = alignment;
        Box::new(self)
    }
}

impl<T: State<T=f64> + Clone> OtherEventHandler for VSplit<T> {
    fn handle_other_event(&mut self, _event: &WidgetEvent, env: &mut Environment) {
        if self.dragging || self.hovering {
            env.set_cursor(MouseCursor::RowResize);
        }
    }
}

impl<T: State<T=f64> + Clone> MouseEventHandler for VSplit<T> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {
        let press_margin = 5.0;

        match event {
            MouseEvent::Press(_, position, _) => {
                let relative_to_position = *position - self.position;

                let split = self.children[0].dimension();

                if relative_to_position.y > split.height - press_margin
                    && relative_to_position.y < split.height + press_margin
                    && relative_to_position.x > 0.0
                    && relative_to_position.x < self.width()
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
                if relative_to_position.y > split.height - press_margin
                    && relative_to_position.y < split.height + press_margin
                    && relative_to_position.x > 0.0
                    && relative_to_position.x < self.width()
                {
                    self.hovering = true;
                } else {
                    self.hovering = false;
                }

                if !self.dragging {
                    return;
                }

                let height = self.height();

                match &mut self.split {
                    SplitType::Start(offset) => {
                        let new_offset = relative_to_position.y;
                        offset.set_value(new_offset.max(0.0).min(height));
                    }
                    SplitType::Percent(percent) => {
                        let p = relative_to_position.y / height;
                        percent.set_value(p.max(0.0).min(1.0));
                    }
                    SplitType::End(offset) => {
                        let new_offset = height - relative_to_position.y;
                        offset.set_value(new_offset.max(0.0).min(height));
                    }
                }
            }
            _ => (),
        }
    }
}

impl<T: State<T=f64> + Clone> Layout for VSplit<T> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let (requested_top_height, requested_bottom_height) = match &self.split {
            SplitType::Start(offset) => (*offset.value(), requested_size.height - *offset.value()),
            SplitType::Percent(percent) => {
                let leading = requested_size.height * *percent.value();
                let trailing = requested_size.height * (1.0 - *percent.value());
                (leading, trailing)
            }
            SplitType::End(offset) => (requested_size.height - *offset.value(), *offset.value()),
        };

        let top_size = Dimension::new(requested_size.width, requested_top_height);
        let mut top = self.children[0].calculate_size(top_size, env);

        let bottom_size = Dimension::new(requested_size.width, requested_bottom_height);
        let mut bottom = self.children[1].calculate_size(bottom_size, env);

        if top.height > requested_top_height {
            let bottom_size =
                Dimension::new(requested_size.width, requested_size.height - top.height);
            bottom = self.children[1].calculate_size(bottom_size, env);
        } else if bottom.height > requested_bottom_height {
            let top_size =
                Dimension::new(requested_size.width, requested_size.height - bottom.height);
            top = self.children[0].calculate_size(top_size, env);
        }

        self.set_dimension(Dimension::new(
            top.width.max(bottom.width),
            requested_size.height,
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
                CrossAxisAlignment::Start => position.x,
                CrossAxisAlignment::Center => {
                    position.x + dimension.width / 2.0 - child.dimension().width / 2.0
                }
                CrossAxisAlignment::End => position.x + dimension.width - child.dimension().width,
            };

            child.set_position(Position::new(cross, position.y + main_axis_offset));
            main_axis_offset += child.dimension().height;
            child.position_children(env);
        });
    }
}

impl<T: State<T=f64> + Clone> CommonWidget for VSplit<T> {
    CommonWidgetImpl!(self, id: self.id, child: self.children, position: self.position, dimension: self.dimension);
}

impl<T: State<T=f64> + Clone> WidgetExt for VSplit<T> {}
