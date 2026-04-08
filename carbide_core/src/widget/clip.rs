use crate::scene::SceneManager;

use crate::CommonWidgetImpl;
use crate::draw::{Alignment, Dimension, Position, Rect};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::widget::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Clip<W>
where
    W: Widget
{
    #[id] id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
    alignment: Alignment,
}

impl Clip<Empty> {
    pub fn new<W: Widget>(child: W) -> Clip<W> {
        Clip {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            alignment: Alignment::Center,
        }
    }
}

impl<W: Widget> Clip<W> {
    pub fn alignment(mut self, alignment: Alignment) -> Clip<W> {
        self.alignment = alignment;
        self
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

    fn position_children(&mut self, bounding_box: Rect, ctx: &mut LayoutContext) {
        let positioning = self.alignment;
        let position = self.position();
        let dimension = self.dimension();
        let inner_bounding_box = self.bounding_box();

        if self.child_count() != 0 {
            let child = self.child(0);
            child.set_position(positioning.position(position, dimension, child.dimension()));
            child.position_children(
                inner_bounding_box.within_bounding_box(&bounding_box),
                ctx
            );
        }
    }
}

impl<W: Widget> CommonWidget for Clip<W> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, alignment: self.alignment);
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

        if let Some(scene_dimensions) = ctx.env.get_mut::<SceneManager>().map(|a| a.dimensions()) {
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
            if self.child_count() != 0 {
                self.child(0).render(this);
            }
        });
    }
}