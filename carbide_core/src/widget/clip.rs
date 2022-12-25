use carbide_macro::carbide_default_builder;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::layout::Layout;
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::widget::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Clip {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
}

impl Clip {
    #[carbide_default_builder]
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {}

    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Clip {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

impl Layout for Clip {
    // Calculate the size of the child, but force clip to requested_size. This makes sure that if
    // the child is larger than the requested, that is is clipped.
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.child.calculate_size(requested_size, env);
        self.dimension = requested_size;
        requested_size
    }
}

CommonWidgetImpl!(Clip, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl Render for Clip {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        // Cut the rendering if either the width or the height is 0
        let min = 1.0 / env.scale_factor();
        if self.dimension.width <= min || self.dimension.height <= min {
            return;
        }

        // If the clip is completely out of frame
        if self.position.x + self.dimension.width < 0.0 {
            return;
        }
        if self.position.y + self.dimension.height < 0.0 {
            return;
        }
        if self.position.x >= env.current_window_width() {
            return;
        }
        if self.position.y >= env.current_window_height() {
            return;
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::Clip,
            bounding_box: Rect::new(self.position, self.dimension),
        });

        for mut child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::UnClip,
            bounding_box: Rect::new(self.position, self.dimension),
        });
    }
}

impl WidgetExt for Clip {}
