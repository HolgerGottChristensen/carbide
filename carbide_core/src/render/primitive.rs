use crate::draw::{BoundingBox, Rect};
use crate::render::primitive_kind::PrimitiveKind;

/// Data required for rendering a single primitive widget.
pub struct Primitive {
    /// State and style for this primitive widget.
    pub kind: PrimitiveKind,
    /// The bounding box of the primitive.
    pub bounding_box: BoundingBox,
}
