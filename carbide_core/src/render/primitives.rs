use carbide_core::draw::Position;

use crate::draw::Dimension;
use crate::environment::Environment;
use crate::layout::Layouter;
use crate::render::primitive::Primitive;
use crate::widget::AnyWidget;

pub struct Primitives {
    pub primitives: Vec<Primitive>,
}

impl Primitives {
    pub fn new(
        window_dimensions: Dimension,
        root: &mut Box<dyn AnyWidget>,
        environment: &mut Environment,
    ) -> Vec<Primitive> {
        todo!()
        // root.calculate_size(window_dimensions, environment);
        //
        // let layout = environment.root_alignment();
        // (layout.positioner())(Position::new(0.0, 0.0), window_dimensions, root);
        //
        // root.position_children(environment);
        // let mut prims: Vec<Primitive> = vec![];
        // root.process_get_primitives(&mut prims, environment);
        // prims
    }
}
