use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::render::primitive_walker::PrimitiveWalker;
use crate::state::environment::Environment;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::render::Render;

pub struct CPrimitives {
    pub primitives: Vec<Primitive>
}

impl CPrimitives {
    pub fn new<S>(window_dimensions: Dimensions, root: &mut Box<dyn Widget<S>>, environment: &mut Environment) -> Self {
        root.calculate_size(window_dimensions, environment);

        root.set_x(window_dimensions[0] / 2.0 - root.get_width() / 2.0);
        root.set_y(window_dimensions[1]/2.0-root.get_height()/2.0);

        root.position_children();

        let prims: Vec<Primitive> = root.get_primitives(environment.get_fonts_map());
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
        }
    }
}