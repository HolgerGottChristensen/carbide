use crate::Rect;
use crate::render::owned_primitive_kind::OwnedPrimitiveKind;
use crate::widget::old::id::Id;

#[derive(Clone)]
pub struct OwnedPrimitive {
    pub id: Id,
    pub kind: OwnedPrimitiveKind,
    pub scizzor: Rect,
    pub rect: Rect,
}

