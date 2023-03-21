use cgmath::Matrix4;
pub use primitive::*;
pub use primitive_kind::*;
pub use primitives::*;
pub use render::*;
pub use render_context::*;
pub use style::*;

use crate::draw::Rect;

mod primitive;
mod primitive_kind;
mod primitives;
mod render;
mod render_context;
mod style;

pub type CarbideTransform = Matrix4<f32>;

/// Simplify the constructor for a `Primitive`.
pub fn new_primitive(kind: PrimitiveKind, rect: Rect) -> Primitive {
    Primitive {
        kind,
        bounding_box: rect,
    }
}
