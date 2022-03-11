use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Clip {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
}

impl Clip {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Clip {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }

    /*pub fn body(&mut self) -> Box<Self> {
        widget_body!(
            HStack (
                alignment: Alignment::Top,
                spacing: 10.0,
            ) {
                for i in $self.model {
                    Text("Item: {}, index: {}", $item, $index),
                }
            }
        )
    }*/
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
        if self.dimension.width == 0.0 || self.dimension.height == 0.0 {
            return
        }

        // If the clip is completely out of frame
        if self.position.x + self.dimension.width < 0.0 {
            return
        }
        if self.position.y + self.dimension.height < 0.0 {
            return
        }
        if self.position.x >= env.get_corrected_width() {
            return
        }
        if self.position.y >= env.get_corrected_height() {
            return
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::Clip,
            rect: Rect::new(self.position, self.dimension),
        });

        for mut child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::UnClip,
            rect: Rect::new(self.position, self.dimension),
        });
    }
}

impl WidgetExt for Clip {}
