use std::fmt;
use crate::{Font, GlyphId, Scale, vector};
use crate::scaled_glyph::ScaledGlyph;

/// A single glyph of a font.
///
/// A `Glyph` does not have an inherent scale or position associated with it. To
/// augment a glyph with a size, give it a scale using `scaled`. You can then
/// position it using `positioned`.
#[derive(Clone)]
pub struct Glyph<'font> {
    pub(crate) font: Font<'font>,
    pub(crate) id: GlyphId,
}

impl<'font> Glyph<'font> {
    /// The font to which this glyph belongs.
    pub fn font(&self) -> &Font<'font> {
        &self.font
    }

    /// The glyph identifier for this glyph.
    pub fn id(&self) -> GlyphId {
        self.id
    }

    /// Augments this glyph with scaling information, making methods that depend
    /// on the scale of the glyph available.
    pub fn scaled(self, scale: Scale) -> ScaledGlyph<'font> {
        let scale_y = self.font.scale_for_pixel_height(scale.y);
        let scale_x = scale_y * scale.x / scale.y;
        ScaledGlyph {
            g: self,
            api_scale: scale,
            scale: vector(scale_x, scale_y),
        }
    }
}

impl fmt::Debug for Glyph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Glyph").field("id", &self.id().0).finish()
    }
}
