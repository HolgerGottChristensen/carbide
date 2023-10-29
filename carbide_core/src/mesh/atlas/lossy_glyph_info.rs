use carbide_rusttype::GlyphId;

use crate::text::{FontId};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LossyGlyphInfo {
    pub(crate) font_id: FontId,
    pub(crate) glyph_id: GlyphId,
    pub(crate) font_size: u32,
}