use render::primitive_walker::PrimitiveWalker;
use render::primitive::Primitive;
use widget::primitive::Widget;
use widget::render::Render;
use render::owned_primitive::OwnedPrimitive;
use render::owned_primitive_kind::OwnedPrimitiveKind;
use render::primitive_kind::PrimitiveKind;
use render::owned_text::OwnedText;
use render::text::Text;
use text;
use position::Dimensions;
use widget::common_widget::CommonWidget;
use widget::layout::Layout;

pub struct CPrimitives {
    primitives: Vec<Primitive>
}

impl CPrimitives {
    pub fn new (window_dimensions: Dimensions, root: &mut Box<dyn Widget>, fonts: &text::font::Map) -> Self {
        root.calculate_size(window_dimensions, fonts);

        root.set_x(window_dimensions[0]/2.0-root.get_width()/2.0);
        root.set_y(window_dimensions[1]/2.0-root.get_height()/2.0);

        root.position_children();

        let mut prims: Vec<Primitive> = root.get_primitives(fonts);
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