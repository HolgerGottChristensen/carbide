use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;
use crate::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct ClipShape {
    id: Uuid,
    child: Box<dyn Widget>,
    shape: Box<dyn Shape>,
    position: Position,
    dimension: Dimension,
}

impl ClipShape {
    pub fn new(child: Box<dyn Widget>, shape: Box<dyn Shape>) -> Box<Self> {
        Box::new(ClipShape {
            id: Uuid::new_v4(),
            child,
            shape,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

impl Layout for ClipShape {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.child.calculate_size(requested_size, env);
        self.shape.calculate_size(requested_size, env);
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.positioner();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);
        positioning(position, dimension, &mut self.shape);

        self.child.position_children();
        self.shape.position_children();
    }
}

CommonWidgetImpl!(ClipShape, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);

impl Render for ClipShape {
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
