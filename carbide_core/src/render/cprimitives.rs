use instant::Instant;

use crate::draw::Dimension;
use crate::prelude::Environment;
use crate::render::primitive::Primitive;
use crate::render::primitive_walker::PrimitiveWalker;
use crate::render::render::RenderProcessor;
use crate::widget::Widget;

pub struct CPrimitives {
    pub primitives: Vec<Primitive>,
}

impl CPrimitives {
    pub fn new(window_dimensions: Dimension, root: &mut Box<dyn Widget>, environment: &mut Environment) -> Self {
        let now = Instant::now();
        root.calculate_size(window_dimensions, environment);

        root.set_x(window_dimensions.width / 2.0 - root.width() / 2.0);
        root.set_y(window_dimensions.height / 2.0 - root.height() / 2.0);

        root.position_children();
        println!("Time for pos and size: {:?}us", now.elapsed().as_micros());
        let mut prims: Vec<Primitive> = vec![];
        root.process_get_primitives(&mut prims, environment);
        println!("Number of primitives: {}", prims.len());
        CPrimitives {
            primitives: prims
        }
    }
}

impl PrimitiveWalker for CPrimitives {
    fn next_primitive(&mut self) -> Option<Primitive> {
        return if !self.primitives.is_empty() {
            Some(self.primitives.remove(0))
        } else {
            None
        };
    }
}