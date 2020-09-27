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

pub struct CPrimitives {
    primitives: Vec<Primitive>
}

impl CPrimitives {
    pub fn new (root: &CWidget, fonts: &text::font::Map) -> Self {
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