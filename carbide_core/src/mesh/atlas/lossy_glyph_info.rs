use rusttype::GlyphId;

use crate::draw::Position;
use crate::text::{FontId, FontSize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LossyGlyphInfo {
    pub(crate) font_id: FontId,
    pub(crate) glyph_id: GlyphId,
    /// Normalised subpixel positions divided by `position_tolerance` & rounded
    ///
    /// `u16` is enough as subpixel position `[-0.5, 0.5]` converted to `[0, 1]`
    ///  divided by the min `position_tolerance` (`0.001`) is small.
    pub(crate) offset_over_tolerance: (u16, u16),
    pub(crate) font_size: u32,
}

impl LossyGlyphInfo {
    pub(crate) fn new(font_id: FontId, glyph_id: GlyphId, font_size: FontSize, offset_over_tolerance: (u16, u16)) -> LossyGlyphInfo {
        LossyGlyphInfo {
            font_id,
            glyph_id,
            offset_over_tolerance,
            font_size,
        }
    }
}
