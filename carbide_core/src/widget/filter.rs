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
    filter_id: Option<FilterId>,
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
            filter_id: None,
        }
    }
}

impl<W: Widget> CommonWidget for Filter<W> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<W: Widget> Render for Filter<W> {
    fn render(&mut self, context: &mut RenderContext) {
        let filter_id = if let Some(filter_id) = self.filter_id {
            filter_id
        } else {
            let id = context.env.insert_filter(self.filter.clone());
            self.filter_id = Some(id);
            id
        };

        context.filter(filter_id, Rect::new(self.position, self.dimension), |this| {
            self.foreach_child_mut(&mut |child| {
                child.render(this);
            });
        });
    }
}