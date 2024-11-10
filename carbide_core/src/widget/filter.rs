use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::render::{Render, RenderContext};
use crate::widget::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Filter<W> where W: Widget {
    id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
    filter: ImageFilter,
}

impl Filter<Empty> {
    #[carbide_default_builder2]
    pub fn new<W: Widget>(filter: ImageFilter, child: W) -> Filter<W> {
        Filter {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            filter,
        }
    }
}

impl<W: Widget> CommonWidget for Filter<W> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<W: Widget> Render for Filter<W> {
    fn render(&mut self, context: &mut RenderContext) {
        context.filter(&self.filter, Rect::new(self.position, self.dimension), |this| {
            self.child.foreach_mut(&mut |child| {
                child.render(this);
            });
        });
    }
}