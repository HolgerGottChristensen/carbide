use carbide::layout::LayoutContext;
use carbide_core::environment::Environment;
use carbide_core::state::IntoReadState;

use carbide_macro::{carbide_default_builder2};

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::state::{ReadState};
use crate::widget::{CommonWidget, Empty, WidgetExt, WidgetId, Widget};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Offset<X, Y, C> where X: ReadState<T=f64>, Y: ReadState<T=f64>, C: Widget {
    id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    #[state] offset_x: X,
    #[state] offset_y: Y,
}

impl Offset<f64, f64, Empty> {
    #[carbide_default_builder2]
    pub fn new<X: IntoReadState<f64>, Y: IntoReadState<f64>, C: Widget>(offset_x: X, offset_y: Y, child: C) -> Offset<X::Output, Y::Output, C> {
        Offset {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            offset_x: offset_x.into_read_state(),
            offset_y: offset_y.into_read_state(),
        }
    }
}

impl<X: ReadState<T=f64>, Y: ReadState<T=f64>, C: Widget> Layout for Offset<X, Y, C> {
    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let positioning = BasicLayouter::Center.positioner();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);

        let mut child_position = self.child.position();

        child_position.x += *self.offset_x.value();
        child_position.y += *self.offset_y.value();

        self.child.set_position(child_position);

        self.child.position_children(ctx);
    }
}

impl<X: ReadState<T=f64>, Y: ReadState<T=f64>, C: Widget> CommonWidget for Offset<X, Y, C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<X: ReadState<T=f64>, Y: ReadState<T=f64>, C: Widget> WidgetExt for Offset<X, Y, C> {}
