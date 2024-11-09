use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Angle, Dimension, Position};
use crate::render::{Render, RenderContext};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct HueRotation<C, S>
where
    C: Widget,
    S: ReadState<T=Angle>,
{
    id: WidgetId,
    child: C,
    #[state] rotation: S,
    position: Position,
    dimension: Dimension,
}

impl HueRotation<Empty, Angle> {
    #[carbide_default_builder2]
    pub fn new<C: Widget, S: IntoReadState<Angle>>(child: C, rotation: S) -> HueRotation<C, S::Output> {
        HueRotation {
            id: WidgetId::new(),
            child,
            rotation: rotation.into_read_state(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<C: Widget, S: ReadState<T=Angle>> CommonWidget for HueRotation<C, S> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: Widget, S: ReadState<T=Angle>> Render for HueRotation<C, S> {
    fn render(&mut self, context: &mut RenderContext) {
        self.rotation.sync(context.env_stack);

        let angle = *self.rotation.value();

        context.hue_rotation(angle.turns() as f32, |this| {
            self.child.render(this)
        })
    }
}