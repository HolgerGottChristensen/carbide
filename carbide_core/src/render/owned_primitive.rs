use crate::OldRect;
use crate::render::owned_primitive_kind::OwnedPrimitiveKind;

#[derive(Clone)]
pub struct OwnedPrimitive {
    pub kind: OwnedPrimitiveKind,
    pub rect: OldRect,
}

