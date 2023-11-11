use carbide_macro::{carbide_default_builder2};

use crate::cursor::MouseCursor;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent};
use crate::layout::{Layout, LayoutContext};
use crate::state::{IntoState, State};
use crate::widget::{CommonWidget, CrossAxisAlignment, SplitType, AnyWidget, WidgetExt, WidgetId, Widget, Empty, WidgetSequence};

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout, MouseEvent, OtherEvent)]
pub struct VSplit<S, L, T> where S: State<T=f64>, L: Widget, T: Widget {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    leading: L,
    trailing: T,
    split: SplitType<S>,
    cross_axis_alignment: CrossAxisAlignment,
    dragging: bool,
    hovering: bool,
    draggable: bool,
}

impl VSplit<f64, Empty, Empty> {

    #[carbide_default_builder2]
    pub fn new<L: Widget, T: Widget>(leading: L, trailing: T) -> VSplit<f64, L, T> {
        Self::new_internal(leading, trailing, SplitType::Percent(0.1), true)
    }
}

impl<S: State<T=f64>, L: Widget, T: Widget> VSplit<S, L, T> {

    fn new_internal<S2: State<T=f64>, L2: Widget, T2: Widget>(
        leading: L2,
        trailing: T2,
        split: SplitType<S2>,
        draggable: bool,
    ) -> VSplit<S2, L2, T2> {
        VSplit {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            leading,
            trailing,
            split,
            cross_axis_alignment: CrossAxisAlignment::Center,
            dragging: false,
            hovering: false,
            draggable,
        }
    }

    pub fn relative_to_start<S2: IntoState<f64>>(self, width: S2) -> VSplit<S2::Output, L, T> {
        Self::new_internal(
            self.leading,
            self.trailing,
            SplitType::Start(width.into_state()),
            self.draggable,
        )
    }

    pub fn percent<S2: IntoState<f64>>(self, percent: S2) -> VSplit<S2::Output, L, T> {
        Self::new_internal(
            self.leading,
            self.trailing,
            SplitType::Percent(percent.into_state()),
            self.draggable,
        )
    }

    pub fn relative_to_end<S2: IntoState<f64>>(self, width: S2) -> VSplit<S2::Output, L, T> {
        Self::new_internal(
            self.leading,
            self.trailing,
            SplitType::End(width.into_state()),
            self.draggable,
        )
    }

    pub fn non_draggable(self) -> Self {
        Self::new_internal(
            self.leading,
            self.trailing,
            self.split,
            false,
        )
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }
}

impl<S: State<T=f64>, L: Widget, T: Widget> OtherEventHandler for VSplit<S, L, T> {
    fn handle_other_event(&mut self, _event: &WidgetEvent, env: &mut Environment) {
        if self.dragging || self.hovering {
            env.set_cursor(MouseCursor::RowResize);
        }
    }
}

impl<S: State<T=f64>, L: Widget, T: Widget> MouseEventHandler for VSplit<S, L, T> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {
        let press_margin = 5.0;

        match event {
            MouseEvent::Press(_, position, _) => {
                let relative_to_position = *position - self.position;

                let split = self.leading.dimension();

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

                let split = self.leading.dimension();
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

impl<S: State<T=f64>, L: Widget, T: Widget> Layout for VSplit<S, L, T> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
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
        let mut top = self.leading.calculate_size(top_size, ctx);

        let bottom_size = Dimension::new(requested_size.width, requested_bottom_height);
        let mut bottom = self.trailing.calculate_size(bottom_size, ctx);

        if top.height > requested_top_height {
            let bottom_size =
                Dimension::new(requested_size.width, requested_size.height - top.height);
            bottom = self.trailing.calculate_size(bottom_size, ctx);
        } else if bottom.height > requested_bottom_height {
            let top_size =
                Dimension::new(requested_size.width, requested_size.height - bottom.height);
            top = self.leading.calculate_size(top_size, ctx);
        }

        self.set_dimension(Dimension::new(
            top.width.max(bottom.width),
            requested_size.height,
        ));
        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
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
            child.position_children(ctx);
        });
    }
}

impl<S: State<T=f64>, L: Widget, T: Widget> CommonWidget for VSplit<S, L, T> {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        self.leading.foreach(f);
        self.trailing.foreach(f);
    }
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.leading.foreach_mut(f);
        self.trailing.foreach_mut(f);
    }
    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.leading.foreach_rev(f);
        self.trailing.foreach_rev(f);
    }
    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.leading.foreach_direct(f);
        self.trailing.foreach_direct(f);
    }
    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.leading.foreach_direct_rev(f);
        self.trailing.foreach_direct_rev(f);
    }
    fn position(&self) -> Position {
        self.position
    }
    fn set_position(&mut self, position: Position) {
        self.position = position;
    }
    fn dimension(&self) -> Dimension {
        self.dimension
    }
    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<S: State<T=f64>, L: Widget, T: Widget> WidgetExt for VSplit<S, L, T> {}
