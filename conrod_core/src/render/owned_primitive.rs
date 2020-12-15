use widget::old::id::Id;
use Rect;
use render::owned_primitive_kind::OwnedPrimitiveKind;

#[derive(Clone)]
pub struct OwnedPrimitive {
    pub id: Id,
    pub kind: OwnedPrimitiveKind,
    pub scizzor: Rect,
    pub rect: Rect,
}

