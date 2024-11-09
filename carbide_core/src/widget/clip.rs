use carbide::scene::SceneManager;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
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
    pub fn new<W: Widget>(child: W) -> Clip<W> {
        Clip {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<W: Widget> Layout for Clip<W> {
    // Calculate the size of the child, but force clip to requested_size. This makes sure that if
    // the child is larger than the requested, that is is clipped.
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.child.calculate_size(requested_size, ctx);
        self.dimension = requested_size;
        requested_size
    }
}

impl<W: Widget> CommonWidget for Clip<W> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<W: Widget> Render for Clip<W> {
    fn render(&mut self, ctx: &mut RenderContext) {
        // If the clip is completely out of frame
        if self.position.x + self.dimension.width < 0.0 {
            return;
        }
        if self.position.y + self.dimension.height < 0.0 {
            return;
        }

        if let Some(scene_dimensions) = ctx.env_stack.get_mut::<SceneManager>().map(|a| a.dimensions()) {
            if self.position.x >= scene_dimensions.width {
                return;
            }
            if self.position.y >= scene_dimensions.height {
                return;
            }
        }

        if self.dimension.width < 1.0 || self.dimension.height < 1.0 {
            return;
        }

        ctx.clip(Rect::new(self.position, self.dimension), |this| {
            self.foreach_child_mut(&mut |child| {
                child.render(this);
            });
        })
    }
}