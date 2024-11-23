use carbide::widget::AnyWidget;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::render::{Render, RenderContext};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Luminance<C, S>
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

impl Luminance<Empty, f64> {
    #[carbide_default_builder2]
    pub fn new<C: Widget, S: IntoReadState<f64>>(child: C, shift: S) -> Luminance<C, S::Output> {
        Luminance {
            id: WidgetId::new(),
            child,
            shift: shift.into_read_state(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<C: Widget, S: ReadState<T=f64>> CommonWidget for Luminance<C, S> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: Widget, S: ReadState<T=f64>> Render for Luminance<C, S> {
    fn render(&mut self, context: &mut RenderContext) {
        self.shift.sync(context.env_stack);

        context.luminance_shift(*self.shift.value() as f32, |this| {
            self.child.render(this)
        })
    }
}