use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Alignment, Dimension, Position};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

/// Takes a child and a background widget, and sizes the background the same as the child.
/// The background will be shown behind the child widget.
///
/// # Examples
/// The example will show some text, with a filled rectangle in the background
/// ```
/// use carbide_core::widget::{Rectangle, Text, WidgetExt};
///
/// Text::new("Hello world")
///     .background(Rectangle::new());
/// ```
///

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout, Render)]
pub struct Background<F, B> where
    F: Widget,
    B: Widget
{
    id: WidgetId,
    child: F,
    background: B,
    position: Position,
    dimension: Dimension,
    alignment: Alignment,
}

impl Background<Empty, Empty> {
    #[carbide_default_builder2]
    pub fn new<F: Widget, B: Widget>(child: F, background: B) -> Background<F, B> {
        Background {
            id: WidgetId::new(),
            child,
            background,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            alignment: Alignment::Center,
        }
    }
}

impl<F: Widget, B: Widget> Background<F, B> {
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl<F: Widget, B: Widget> Layout for Background<F, B> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let child_size = self.child.calculate_size(requested_size, ctx);
        self.background.calculate_size(child_size, ctx);
        self.dimension = child_size;
        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let alignment = self.alignment();
        let position = self.position;
        let dimension = self.dimension;

        self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
        self.background.set_position(alignment.position(position, dimension, self.background.dimension()));

        self.child.position_children(ctx);
        self.background.position_children(ctx);
    }
}

impl<F: Widget, B: Widget> Render for Background<F, B> {
    fn render(&mut self, context: &mut RenderContext) {
        self.background.render(context);
        self.child.render(context);
    }
}

impl<F: Widget, B: Widget> CommonWidget for Background<F, B> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, alignment: self.alignment);
}
