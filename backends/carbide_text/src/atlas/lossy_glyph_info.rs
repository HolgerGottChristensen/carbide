use carbide_rusttype::GlyphId;

use carbide_core::text::{FontId};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LossyGlyphInfo {
    pub font_id: FontId,
    pub glyph_id: GlyphId,
    pub font_size: u32,
}