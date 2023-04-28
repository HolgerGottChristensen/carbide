use carbide_core::render::RenderContext;
use carbide_core::widget::CommonWidget;

use carbide_macro::carbide_default_builder;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::render::{Primitive, Render};
use crate::widget::{Empty, Widget, WidgetExt, WidgetId};

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
    F: Widget + Clone,
    B: Widget + Clone
{
    id: WidgetId,
    child: F,
    background: B,
    position: Position,
    dimension: Dimension,
    alignment: Box<dyn Layouter>,
}

impl Background<Empty, Empty> {
    #[carbide_default_builder]
    pub fn new<F: Widget + Clone, B: Widget + Clone>(child: F, background: B) -> Box<Background<F, B>> {}

    pub fn new<F: Widget + Clone, B: Widget + Clone>(child: F, background: B) -> Box<Background<F, B>> {
        Box::new(Background {
            id: WidgetId::new(),
            child,
            background,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            alignment: Box::new(BasicLayouter::Center),
        })
    }
}

impl<F: Widget + Clone, B: Widget + Clone> Background<F, B> {
    pub fn with_alignment(mut self, layouter: BasicLayouter) -> Box<Self> {
        self.alignment = Box::new(layouter);
        Box::new(self)
    }
}

impl<F: Widget + Clone, B: Widget + Clone> Layout for Background<F, B> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let child_size = self.child.calculate_size(requested_size, env);
        self.background.calculate_size(child_size, env);
        self.dimension = child_size;
        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
        let positioning = self.alignment.positioner();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);
        positioning(position, dimension, &mut self.background);
        self.child.position_children(env);
        self.background.position_children(env);
    }
}

impl<F: Widget + Clone, B: Widget + Clone> Render for Background<F, B> {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.background.process_get_primitives(primitives, env);
        self.child.process_get_primitives(primitives, env);
    }

    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.background.render(context, env);
        self.child.render(context, env);
    }
}

impl<F: Widget + Clone, B: Widget + Clone> CommonWidget for Background<F, B> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, alignment: self.alignment);
}

impl<F: Widget + Clone, B: Widget + Clone> WidgetExt for Background<F, B> {}
