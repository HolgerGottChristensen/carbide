use crate::{Point, text};
use crate::draw::shape::triangle::Triangle;
use crate::render::owned_primitive::OwnedPrimitive;
use crate::render::walk_owned_primitives::WalkOwnedPrimitives;
use crate::widget::primitive::ColoredPoint;

/// An owned alternative to the `Primitives` type.
///
/// This is particularly useful for sending rendering data across threads.
///
/// Produce an `OwnedPrimitives` instance via the `Primitives::owned` method.
#[derive(Clone)]
pub struct OwnedPrimitives {
    pub(crate) primitives: Vec<OwnedPrimitive>,
    pub(crate) triangles_single_color: Vec<Triangle<Point>>,
    pub(crate) triangles_multi_color: Vec<Triangle<ColoredPoint>>,
    pub(crate) max_glyphs: usize,
    pub(crate) line_infos: Vec<text::line::Info>,
    pub(crate) texts_string: String,
}


impl OwnedPrimitives {

    /// Produce an iterator-like type for yielding `Primitive`s.
    pub fn walk(&self) -> WalkOwnedPrimitives {
        let OwnedPrimitives {
            ref primitives,
            ref triangles_single_color,
            ref triangles_multi_color,
            ref line_infos,
            ref texts_string,
            max_glyphs,
        } = *self;
        WalkOwnedPrimitives {
            primitives: primitives.iter(),
            triangles_single_color: triangles_single_color,
            triangles_multi_color: triangles_multi_color,
            line_infos: line_infos,
            texts_str: texts_string,
            positioned_glyphs: Vec::with_capacity(max_glyphs),
        }
    }

}

