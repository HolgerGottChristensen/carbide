use carbide::environment::Environment;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Alignment, Dimension, Position};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::widget::{CommonWidget, Empty, AnyShape, Widget, WidgetId, WidgetSync, AnyWidget, ShapeStyle};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, StateSync)]
pub struct ClipShape<C, S>
where
    C: Widget,
    S: AnyShape + AnyWidget + Clone
{
    #[id] id: WidgetId,
    child: C,
    shape: S,
    position: Position,
    dimension: Dimension,
}

impl ClipShape<Empty, Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget, S: AnyShape + AnyWidget + Clone>(child: C, shape: S) -> ClipShape<C, S> {
        ClipShape {
            id: WidgetId::new(),
            child,
            shape,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<C: Widget, S: AnyShape + AnyWidget + Clone> WidgetSync for ClipShape<C, S> {
    fn sync(&mut self, env: &mut Environment) {
        self.child.sync(env);
        self.shape.sync(env);
    }
}

impl<C: Widget, S: AnyShape + AnyWidget + Clone> Layout for ClipShape<C, S> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.child.calculate_size(requested_size, ctx);
        self.shape.calculate_size(requested_size, ctx);
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let position = self.position;
        let dimension = self.dimension;

        self.child.set_position(Alignment::Center.position(position, dimension, self.child.dimension()));
        self.shape.set_position(Alignment::Center.position(position, dimension, self.shape.dimension()));

        self.child.position_children(ctx);
        self.shape.position_children(ctx);
    }
}

impl<C: Widget, S: AnyShape + AnyWidget + Clone> CommonWidget for ClipShape<C, S> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: Widget, S: AnyShape + AnyWidget + Clone> Render for ClipShape<C, S> {
    fn render(&mut self, context: &mut RenderContext) {
        context.stencil(&self.shape, self.shape.options(), |ctx| {
            self.child.render(ctx)
        })
    }
}