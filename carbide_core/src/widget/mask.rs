use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::layout::{BasicLayouter, Layout, LayoutContext, Layouter};
use crate::render::Render;
use crate::state::StateSync;
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, StateSync)]
pub struct Mask<M, W>
where
    M: Widget,
    W: Widget,
{
    id: WidgetId,
    child: W,
    mask: M,
    position: Position,
    dimension: Dimension,
}

impl Mask<Empty, Empty> {
    #[carbide_default_builder2]
    pub fn new<M: Widget, W: Widget>(mask: M, child: W) -> Mask<M, W> {
        Mask {
            id: WidgetId::new(),
            child,
            mask,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<M: Widget, W: Widget> StateSync for Mask<M, W> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.child.capture_state(env);
        self.mask.capture_state(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.child.release_state(env);
        self.mask.release_state(env);
    }
}

impl<M: Widget, W: Widget> Layout for Mask<M, W> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.child.calculate_size(requested_size, ctx);
        self.mask.calculate_size(requested_size, ctx);
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let positioning = BasicLayouter::Center.positioner();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);
        positioning(position, dimension, &mut self.mask);

        self.child.position_children(ctx);
        self.mask.position_children(ctx);
    }
}

impl<M: Widget, W: Widget> CommonWidget for Mask<M, W> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<M: Widget, W: Widget> Render for Mask<M, W> {
    fn render(&mut self, context: &mut RenderContext) {
        context.mask(|this| {
            self.mask.render(this);
        }, |this| {
            self.child.render(this);
        })
    }
}

impl<M: Widget, W: Widget> WidgetExt for Mask<M, W> {}
