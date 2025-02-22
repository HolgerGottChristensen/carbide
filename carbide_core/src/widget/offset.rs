use carbide_macro::carbide_default_builder2;

use crate::draw::{Alignment, Dimension, Position};
use crate::layout::{Layout, LayoutContext};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetId};
use crate::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Offset<X, Y, C> where X: ReadState<T=f64>, Y: ReadState<T=f64>, C: Widget {
    #[id] id: WidgetId,
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
        let position = self.position;
        let dimension = self.dimension;

        self.child.set_position(Alignment::Center.position(position, dimension, self.child.dimension()));

        let mut child_position = self.child.position();

        child_position.x += *self.offset_x.value();
        child_position.y += *self.offset_y.value();

        self.child.set_position(child_position);

        self.child.position_children(ctx);
    }
}

impl<X: ReadState<T=f64>, Y: ReadState<T=f64>, C: Widget> CommonWidget for Offset<X, Y, C> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}