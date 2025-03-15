use crate::draw::Rect;
use crate::text::glyph::GlyphRenderMode;

#[derive(Debug, Clone)]
pub struct Glyph {
    pub bounding_box: Rect,
    pub texture_coords: Rect,
    pub mode: GlyphRenderMode
}