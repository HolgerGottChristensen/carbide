use crate::Rect;
use crate::render::owned_primitive_kind::OwnedPrimitiveKind;

#[derive(Clone)]
pub struct OwnedPrimitive {
    pub kind: OwnedPrimitiveKind,
    pub rect: Rect,
}

