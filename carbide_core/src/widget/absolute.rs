use carbide::layout::LayoutContext;
use carbide_core::state::IntoReadState;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Scalar};
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::state::ReadState;
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Absolute<X, Y, C> where X: ReadState<T=Scalar>, Y: ReadState<T=Scalar>, C: Widget {
    id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    #[state] x: X,
    #[state] y: Y,
}

impl Absolute<f64, f64, Empty> {
    #[carbide_default_builder2]
    pub fn new<X: IntoReadState<f64>, Y: IntoReadState<f64>, C: Widget>(x: X, y: Y, child: C) -> Absolute<X::Output, Y::Output, C> {
        Absolute {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            x: x.into_read_state(),
            y: y.into_read_state(),
        }
    }
}

impl<X: ReadState<T=Scalar>, Y: ReadState<T=Scalar>, C: Widget> Layout for Absolute<X, Y, C> {
    fn position_children(&mut self, ctx: &mut LayoutContext) {
        self.x.sync(ctx.env);
        self.y.sync(ctx.env);

        let positioning = BasicLayouter::TopLeading.positioner();
        let position = Position::new(*self.x.value(), *self.y.value());
        let dimension = Dimension::new(ctx.env.current_window_width(), ctx.env.current_window_height());

        positioning(position, dimension, &mut self.child);
        self.child.position_children(ctx);
    }
}

impl<X: ReadState<T=Scalar>, Y: ReadState<T=Scalar>, C: Widget> CommonWidget for Absolute<X, Y, C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<X: ReadState<T=Scalar>, Y: ReadState<T=Scalar>, C: Widget> WidgetExt for Absolute<X, Y, C> {}
