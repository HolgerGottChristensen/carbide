use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::render::{Render, RenderContext};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct HueRotation<C, S>
where
    C: Widget,
    S: ReadState<T=f64>,
{
    id: WidgetId,
    child: C,
    #[state] rotation: S,
    position: Position,
    dimension: Dimension,
}

impl HueRotation<Empty, f64> {
    #[carbide_default_builder2]
    pub fn new<C: Widget, S: IntoReadState<f64>>(child: C, rotation: S) -> HueRotation<C, S::Output> {
        HueRotation {
            id: WidgetId::new(),
            child,
            rotation: rotation.into_read_state(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<C: Widget, S: ReadState<T=f64>> CommonWidget for HueRotation<C, S> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: Widget, S: ReadState<T=f64>> Render for HueRotation<C, S> {
    fn render(&mut self, context: &mut RenderContext) {
        self.rotation.sync(context.env);

        context.hue_rotation(*self.rotation.value() as f32, |this| {
            self.child.render(this)
        })
    }
}

impl<C: Widget, S: ReadState<T=f64>> WidgetExt for HueRotation<C, S> {}
