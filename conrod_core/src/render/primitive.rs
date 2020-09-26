use ::{widget, Rect};
use render::primitive_kind::PrimitiveKind;

/// Data required for rendering a single primitive widget.
pub struct Primitive<'a> {
    /// The id of the widget within the widget graph.
    pub id: widget::Id,
    /// State and style for this primitive widget.
    pub kind: PrimitiveKind<'a>,
    /// The Rect to which the primitive widget should be cropped.
    ///
    /// Only parts of the widget within this `Rect` should be drawn.
    pub scizzor: Rect,
    /// The bounding rectangle for the `Primitive`.
    pub rect: Rect,
}

