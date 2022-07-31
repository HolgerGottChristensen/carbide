use crate::draw::Rect;
pub use primitive::*;
pub use primitive_kind::*;
pub use primitives::*;
pub use render::*;

mod primitive;
mod primitive_kind;
mod primitives;
mod render;

/// Simplify the constructor for a `Primitive`.
pub fn new_primitive(kind: PrimitiveKind, rect: Rect) -> Primitive {
    Primitive {
        kind,
        bounding_box: rect,
    }
}
