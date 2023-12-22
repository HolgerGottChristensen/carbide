use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::layout::{Layout, LayoutContext};
use crate::render::Render;
use crate::widget::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Clip<W>
where
    W: Widget
{
    id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
}

impl Clip<Empty> {
    #[carbide_default_builder2]
    pub fn new<W: AnyWidget + Clone>(child: W) -> Clip<W> {
        Clip {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<W: AnyWidget + Clone> Layout for Clip<W> {
    // Calculate the size of the child, but force clip to requested_size. This makes sure that if
    // the child is larger than the requested, that is is clipped.
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.child.calculate_size(requested_size, ctx);
        self.dimension = requested_size;
        requested_size
    }
}

impl<W: AnyWidget + Clone> CommonWidget for Clip<W> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<W: AnyWidget + Clone> Render for Clip<W> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        // If the clip is completely out of frame
        if self.position.x + self.dimension.width < 0.0 {
            return;
        }
        if self.position.y + self.dimension.height < 0.0 {
            return;
        }
        if self.position.x >= env.current_window_width() {
            return;
        }
        if self.position.y >= env.current_window_height() {
            return;
        }

        if self.dimension.width < 1.0 || self.dimension.height < 1.0 {
            return;
        }

        context.clip(Rect::new(self.position, self.dimension), |this| {
            self.foreach_child_mut(&mut |child| {
                child.render(this, env);
            });
        })
    }
}

impl<W: AnyWidget + Clone> WidgetExt for Clip<W> {}
