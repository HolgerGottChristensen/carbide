use std::ops::DerefMut;

use carbide_macro::carbide_default_builder;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::render::{Primitive, Render};
use crate::widget::{Widget, WidgetExt, WidgetId};

/// Takes a child and a background widget, and sizes the background the same as the child.
/// The background will be shown behind the child widget.
///
/// # Examples
/// The example will show some text, with a filled rectangle in the background
/// ```
///     Text::new("Hello world")
///         .background(Rectangle::new())
/// ```

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout, Render)]
pub struct Background {
    id: WidgetId,
    child: Box<dyn Widget>,
    background: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    alignment: Box<dyn Layouter>,
}

impl Background {

    #[carbide_default_builder]
    pub fn new(child: Box<dyn Widget>, background: Box<dyn Widget>) -> Box<Background> {}

    pub fn new(child: Box<dyn Widget>, background: Box<dyn Widget>) -> Box<Background> {
        Box::new(Background {
            id: WidgetId::new(),
            child,
            background,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            alignment: Box::new(BasicLayouter::Center),
        })
    }

    pub fn with_alignment(mut self, layouter: BasicLayouter) -> Box<Self> {
        self.alignment = Box::new(layouter);
        Box::new(self)
    }
}

impl Layout for Background {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let child_size = self.child.calculate_size(requested_size, env);
        self.background.calculate_size(child_size, env);
        self.dimension = child_size;
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = self.alignment.positioner();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, self.child.deref_mut());
        positioning(position, dimension, self.background.deref_mut());
        self.child.position_children();
        self.background.position_children();
    }
}

impl Render for Background {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.background.process_get_primitives(primitives, env);
        self.child.process_get_primitives(primitives, env);
    }
}

CommonWidgetImpl!(Background, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, alignment: self.alignment);

impl WidgetExt for Background {}
