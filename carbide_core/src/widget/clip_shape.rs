use carbide_core::render::RenderContext;

use carbide_macro::{carbide_default_builder, carbide_default_builder2};

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::environment::{Environment, EnvironmentColorState};
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::state::StateSync;
use crate::widget::{CommonWidget, Empty, Rectangle, Shape, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, StateSync)]
pub struct ClipShape<C, S>
where
    C: Widget + Clone,
    S: Shape + Clone
{
    id: WidgetId,
    child: C,
    shape: S,
    position: Position,
    dimension: Dimension,
}

impl ClipShape<Empty, Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget + Clone, S: Shape + Clone>(child: C, shape: S) -> Box<ClipShape<C, S>> {
        Box::new(ClipShape {
            id: WidgetId::new(),
            child,
            shape,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

impl<C: Widget + Clone, S: Shape + Clone> StateSync for ClipShape<C, S> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.child.capture_state(env);
        self.shape.capture_state(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.child.release_state(env);
        self.shape.release_state(env);
    }
}

impl<C: Widget + Clone, S: Shape + Clone> Layout for ClipShape<C, S> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.child.calculate_size(requested_size, env);
        self.shape.calculate_size(requested_size, env);
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self, env: &mut Environment) {
        let positioning = BasicLayouter::Center.positioner();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);
        positioning(position, dimension, &mut self.shape);

        self.child.position_children(env);
        self.shape.position_children(env);
    }
}

impl<C: Widget + Clone, S: Shape + Clone> CommonWidget for ClipShape<C, S> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: Widget + Clone, S: Shape + Clone> Render for ClipShape<C, S> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        let stencil_triangles = &self.shape.triangles(env);

        context.stencil(stencil_triangles, |this| {
            self.foreach_child_mut(&mut |child| {
                child.render(this, env);
            });
        })

    }

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        let stencil_triangles = self.shape.triangles(env);

        primitives.push(Primitive {
            kind: PrimitiveKind::Stencil(stencil_triangles),
            bounding_box: Rect::new(self.position, self.dimension),
        });

        self.foreach_child_mut(&mut |child| {
            child.process_get_primitives(primitives, env);
        });

        primitives.push(Primitive {
            kind: PrimitiveKind::DeStencil,
            bounding_box: Rect::new(self.position, self.dimension),
        });
    }
}

impl<C: Widget + Clone, S: Shape + Clone> WidgetExt for ClipShape<C, S> {}
