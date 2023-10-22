use carbide_core::state::ReadState;
use carbide_macro::{carbide_default_builder, carbide_default_builder2};

use crate::draw::{Dimension, Position};
use crate::environment::Environment;

use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::state::IntoReadState;
use crate::widget::{CommonWidget, Empty, AnyWidget, WidgetExt, WidgetId, Widget};
use crate::widget::types::EdgeInsets;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Padding<W, E> where W: Widget, E: ReadState<T=EdgeInsets> {
    id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
    edge_insets: E,
}

impl Padding<Empty, EdgeInsets> {
    #[carbide_default_builder2]
    pub fn new<W: Widget, E: IntoReadState<EdgeInsets>>(edge_insets: E, child: W) -> Padding<W, E::Output> {
        Padding {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            edge_insets: edge_insets.into_read_state(),
        }
    }
}

impl<W: Widget, E: ReadState<T=EdgeInsets>> CommonWidget for Padding<W, E> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_rev(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        Dimension::new(self.dimension.width.abs(), self.dimension.height.abs())
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<W: Widget, E: ReadState<T=EdgeInsets>> Layout for Padding<W, E> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let insets = *self.edge_insets.value();
        let dimensions = Dimension::new(
            requested_size.width - insets.left - insets.right,
            requested_size.height - insets.top - insets.bottom,
        );

        let child_dimensions = self.child.calculate_size(dimensions, env);

        self.dimension = Dimension::new(
            child_dimensions.width + insets.left + insets.right,
            child_dimensions.height + insets.top + insets.bottom,
        );

        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
        let insets = *self.edge_insets.value();
        let positioning = BasicLayouter::Center.positioner();
        let position = Position::new(
            self.x() + insets.left,
            self.y() + insets.top,
        );
        let dimension = Dimension::new(
            self.width() - insets.left - insets.right,
            self.height() - insets.top - insets.bottom,
        );

        positioning(position, dimension, &mut self.child);
        self.child.position_children(env);
    }
}

impl<W: Widget, E: ReadState<T=EdgeInsets>> WidgetExt for Padding<W, E> {}
