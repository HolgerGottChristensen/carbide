pub use primitives::*;
pub use primitive::*;
pub use primitive_kind::*;
pub use primitive_walker::*;
pub use render::*;
use crate::draw::Rect;

mod primitives;
mod primitive;
mod primitive_kind;
mod primitive_walker;
mod render;

/// Simplify the constructor for a `Primitive`.
pub fn new_primitive(kind: PrimitiveKind, rect: Rect) -> Primitive {
    Primitive { kind, bounding_box: rect }
}
