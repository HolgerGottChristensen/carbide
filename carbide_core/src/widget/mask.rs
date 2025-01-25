use carbide::environment::EnvironmentStack;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Alignment, Dimension, Position};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::state::StateSync;
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetSync};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, StateSync)]
pub struct Mask<M, W>
where
    M: Widget,
    W: Widget,
{
    #[id] id: WidgetId,
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

impl<M: Widget, W: Widget> WidgetSync for Mask<M, W> {
    fn sync(&mut self, env: &mut EnvironmentStack) {
        self.child.sync(env);
        self.mask.sync(env);
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
        let position = self.position;
        let dimension = self.dimension;

        self.child.set_position(Alignment::Center.position(position, dimension, self.child.dimension()));
        self.mask.set_position(Alignment::Center.position(position, dimension, self.mask.dimension()));

        self.child.position_children(ctx);
        self.mask.position_children(ctx);
    }
}

impl<M: Widget, W: Widget> CommonWidget for Mask<M, W> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
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