use carbide_macro::carbide_default_builder2;

use crate::cursor::MouseCursor;
use crate::draw::{Dimension, Position};
use crate::event::{MouseEvent, MouseEventContext, MouseEventHandler};
use crate::layout::{Layout, LayoutContext};
use crate::state::{IntoState, State};
use crate::widget::{AnyWidget, CommonWidget, CrossAxisAlignment, Empty, SplitType, Widget, WidgetExt, WidgetId, WidgetSequence};

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout, MouseEvent)]
pub struct HSplit<S, L, T> where S: State<T=f64>, L: Widget, T: Widget {
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

impl HSplit<f64, Empty, Empty> {

    #[carbide_default_builder2]
    pub fn new<L: Widget, T: Widget>(leading: L, trailing: T) -> HSplit<f64, L, T> {
        Self::new_internal(leading, trailing, SplitType::Percent(0.1), true)
    }
}

impl<S: State<T=f64>, L: Widget, T: Widget> HSplit<S, L, T> {

    fn new_internal<S2: State<T=f64>, L2: Widget, T2: Widget>(
        leading: L2,
        trailing: T2,
        split: SplitType<S2>,
        draggable: bool,
    ) -> HSplit<S2, L2, T2> {
        HSplit {
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

    pub fn relative_to_start<S2: IntoState<f64>>(self, width: S2) -> HSplit<S2::Output, L, T> {
        Self::new_internal(
            self.leading,
            self.trailing,
            SplitType::Start(width.into_state()),
            self.draggable,
        )
    }

    pub fn percent<S2: IntoState<f64>>(self, percent: S2) -> HSplit<S2::Output, L, T> {
        Self::new_internal(
            self.leading,
            self.trailing,
            SplitType::Percent(percent.into_state()),
            self.draggable,
        )
    }

    pub fn relative_to_end<S2: IntoState<f64>>(self, width: S2) -> HSplit<S2::Output, L, T> {
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

impl<S: State<T=f64>, L: Widget, T: Widget> MouseEventHandler for HSplit<S, L, T> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _ctx: &mut MouseEventContext) {
        if !self.draggable {
            return;
        }

        let press_margin = 5.0;

        match event {
            MouseEvent::Press(_, position, _) => {
                let relative_to_position = *position - self.position;

                let split = self.leading.dimension();

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
                let split = self.leading.dimension();

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

impl<S: State<T=f64>, L: Widget, T: Widget> Layout for HSplit<S, L, T> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let (requested_leading_width, requested_trailing_width) = match &self.split {
            SplitType::Start(offset) => (offset.value().clone(), requested_size.width - offset.value().clone()),
            SplitType::Percent(percent) => {
                let leading = requested_size.width * percent.value().clone();
                let trailing = requested_size.width * (1.0 - percent.value().clone());
                (leading, trailing)
            }
            SplitType::End(offset) => (requested_size.width - offset.value().clone(), offset.value().clone()),
        };

        let leading_size = Dimension::new(requested_leading_width, requested_size.height);
        let mut leading = self.leading.calculate_size(leading_size, ctx);

        let trailing_size = Dimension::new(requested_trailing_width, requested_size.height);
        let mut trailing = self.trailing.calculate_size(trailing_size, ctx);

        if leading.width > requested_leading_width {
            let trailing_size =
                Dimension::new(requested_size.width - leading.width, requested_size.height);
            trailing = self.trailing.calculate_size(trailing_size, ctx);
        } else if trailing.width > requested_trailing_width {
            let leading_size =
                Dimension::new(requested_size.width - trailing.width, requested_size.height);
            leading = self.leading.calculate_size(leading_size, ctx);
        }

        self.set_dimension(Dimension::new(
            requested_size.width,
            leading.height.max(trailing.height),
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
                CrossAxisAlignment::Start => position.y,
                CrossAxisAlignment::Center => {
                    position.y + dimension.height / 2.0 - child.dimension().height / 2.0
                }
                CrossAxisAlignment::End => position.y + dimension.height - child.dimension().height,
            };

            child.set_position(Position::new(position.x + main_axis_offset, cross));
            main_axis_offset += child.dimension().width;
            child.position_children(ctx);
        });
    }
}

impl<S: State<T=f64>, L: Widget, T: Widget> CommonWidget for HSplit<S, L, T> {
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

    fn cursor(&self) -> Option<MouseCursor> {
        if self.hovering || self.dragging {
            Some(MouseCursor::ColResize)
        } else {
            None
        }
    }
}

impl<S: State<T=f64>, L: Widget, T: Widget> WidgetExt for HSplit<S, L, T> {}
