use carbide_core::draw::Position;
use crate::draw::Dimension;
use crate::layout::Layouter;
use crate::prelude::Environment;
use crate::render::primitive::Primitive;
use crate::render::primitive_walker::PrimitiveWalker;
use crate::widget::Widget;

pub struct Primitives {
    pub primitives: Vec<Primitive>,
}

impl Primitives {
    pub fn new(
        window_dimensions: Dimension,
        root: &mut Box<dyn Widget>,
        environment: &mut Environment,
    ) -> Self {
        root.calculate_size(window_dimensions, environment);

        let layout = environment.root_alignment();
        (layout.positioner())(Position::new(0.0, 0.0), window_dimensions, root);
        //root.set_x(window_dimensions.width / 2.0 - root.width() / 2.0);
        //root.set_y(window_dimensions.height / 2.0 - root.height() / 2.0);

        root.position_children();
        let mut prims: Vec<Primitive> = vec![];
        root.process_get_primitives(&mut prims, environment);
        Primitives { primitives: prims }
    }
}

impl PrimitiveWalker for Primitives {

    // TODO: This implementation is very inefficient
    fn next_primitive(&mut self) -> Option<Primitive> {
        return if !self.primitives.is_empty() {
            Some(self.primitives.remove(0))
        } else {
            None
        };
    }
}
