use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::render::{Render, RenderContext};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Saturation<C, S>
where
    C: Widget,
    S: ReadState<T=f64>,
{
    #[id] id: WidgetId,
    child: C,
    #[state] shift: S,
    position: Position,
    dimension: Dimension,
}

impl Saturation<Empty, f64> {
    #[carbide_default_builder2]
    pub fn new<C: Widget, S: IntoReadState<f64>>(child: C, shift: S) -> Saturation<C, S::Output> {
        Saturation {
            id: WidgetId::new(),
            child,
            shift: shift.into_read_state(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<C: Widget, S: ReadState<T=f64>> CommonWidget for Saturation<C, S> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: Widget, S: ReadState<T=f64>> Render for Saturation<C, S> {
    fn render(&mut self, context: &mut RenderContext) {
        self.shift.sync(context.env);

        context.saturation_shift(*self.shift.value() as f32, |this| {
            self.child.render(this)
        })
    }
}