use carbide_core::draw::Position;
use crate::draw::Dimension;
use crate::layout::Layouter;
use crate::environment::Environment;
use crate::render::primitive::Primitive;
use crate::widget::Widget;

pub struct Primitives {
    pub primitives: Vec<Primitive>,
}

impl Primitives {
    pub fn new(
        window_dimensions: Dimension,
        root: &mut Box<dyn Widget>,
        environment: &mut Environment,
    ) -> Vec<Primitive> {
        root.calculate_size(window_dimensions, environment);

        let layout = environment.root_alignment();
        (layout.positioner())(Position::new(0.0, 0.0), window_dimensions, root);

        root.position_children();
        let mut prims: Vec<Primitive> = vec![];
        root.process_get_primitives(&mut prims, environment);
        prims
    }
}
