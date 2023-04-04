use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::state::StateSync;
use crate::widget::{CommonWidget, Shape, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, StateSync)]
pub struct ClipShape {
    id: WidgetId,
    child: Box<dyn Widget>,
    shape: Box<dyn Shape>,
    position: Position,
    dimension: Dimension,
}

impl ClipShape {
    #[carbide_default_builder]
    pub fn new(child: Box<dyn Widget>, shape: Box<dyn Shape>) -> Box<Self> {}

    pub fn new(child: Box<dyn Widget>, shape: Box<dyn Shape>) -> Box<Self> {
        Box::new(ClipShape {
            id: WidgetId::new(),
            child,
            shape,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

impl StateSync for ClipShape {
    fn capture_state(&mut self, env: &mut Environment) {
        self.child.capture_state(env);
        self.shape.capture_state(env);
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.child.release_state(env);
        self.shape.release_state(env);
    }
}

impl Layout for ClipShape {
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

CommonWidgetImpl!(ClipShape, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);

impl Render for ClipShape {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        let stencil_triangles = &self.shape.triangles(env);

        context.stencil(stencil_triangles, |this| {
            for mut child in self.children_mut() {
                child.render(this, env);
            }
        })

    }

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        let stencil_triangles = self.shape.triangles(env);

        primitives.push(Primitive {
            kind: PrimitiveKind::Stencil(stencil_triangles),
            bounding_box: Rect::new(self.position, self.dimension),
        });

        for mut child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::DeStencil,
            bounding_box: Rect::new(self.position, self.dimension),
        });
    }
}

impl WidgetExt for ClipShape {}
