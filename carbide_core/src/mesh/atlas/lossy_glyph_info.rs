use carbide_rusttype::GlyphId;

use crate::text::{FontId, FontSize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LossyGlyphInfo {
    pub(crate) font_id: FontId,
    pub(crate) glyph_id: GlyphId,
    pub(crate) font_size: u32,
}

impl LossyGlyphInfo {
    pub(crate) fn new(
        font_id: FontId,
        glyph_id: GlyphId,
        font_size: FontSize,
    ) -> LossyGlyphInfo {
        LossyGlyphInfo {
            font_id,
            glyph_id,
            font_size,
        }
    }
}
