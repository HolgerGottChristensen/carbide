use render::primitive_walker::PrimitiveWalker;
use render::primitive::Primitive;
use widget::primitive::CWidget;
use widget::render::Render;
use render::owned_primitive::OwnedPrimitive;
use render::owned_primitive_kind::OwnedPrimitiveKind;
use render::primitive_kind::PrimitiveKind;
use render::owned_text::OwnedText;
use render::text::Text;
use text;
use position::Dimensions;
use widget::common_widget::CommonWidget;

pub struct CPrimitives {
    primitives: Vec<Primitive>
}

impl CPrimitives {
    pub fn new (window_dimensions: Dimensions, root: &mut CWidget, fonts: &text::font::Map) -> Self {
        root.layout(window_dimensions, fonts, &|c: &mut CommonWidget, dimensions: Dimensions| {
            let new_x = window_dimensions[0]/2.0 - dimensions[0]/2.0;
            let new_y = window_dimensions[1]/2.0 - dimensions[1]/2.0;
            c.set_x(new_x);
            c.set_y(new_y);
        });
        let mut prims: Vec<Primitive> = root.get_primitives(window_dimensions, fonts);
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