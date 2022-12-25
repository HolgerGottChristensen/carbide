use crate::{GlyphId, IntoGlyphId, Scale, VMetrics};
use core::fmt;
use std::sync::Arc;
use owned_ttf_parser::{AsFaceRef, RasterGlyphImage};
use crate::glyph::Glyph;
use crate::glyph_iter::GlyphIter;

#[derive(Clone)]
pub enum Font<'a> {
    Ref(Arc<owned_ttf_parser::Face<'a>>),
    Owned(Arc<owned_ttf_parser::OwnedFace>),
}

impl fmt::Debug for Font<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Font")
    }
}

impl Font<'_> {
    /// Creates a Font from byte-slice data.
    ///
    /// Returns `None` for invalid data.
    pub fn try_from_bytes(bytes: &[u8]) -> Option<Font<'_>> {
        Self::try_from_bytes_and_index(bytes, 0)
    }

    /// Creates a Font from byte-slice data & a font collection `index`.
    ///
    /// Returns `None` for invalid data.
    pub fn try_from_bytes_and_index(bytes: &[u8], index: u32) -> Option<Font<'_>> {
        let inner = owned_ttf_parser::Face::parse(bytes, index);
        match inner {
            Ok(face) => {
                Some(Font::Ref(Arc::new(face)))
            }
            Err(_) => {
                None
            }
        }
    }

    /// Creates a Font from owned font data.
    ///
    /// Returns `None` for invalid data.
    pub fn try_from_vec(data: Vec<u8>) -> Option<Font<'static>> {
        Self::try_from_vec_and_index(data, 0)
    }

    /// Creates a Font from owned font data & a font collection `index`.
    ///
    /// Returns `None` for invalid data.
    pub fn try_from_vec_and_index(data: Vec<u8>, index: u32) -> Option<Font<'static>> {
        let inner = owned_ttf_parser::OwnedFace::from_vec(data, index);
        match inner {
            Ok(owned_face) => {
                Some(Font::Owned(Arc::new(owned_face)))
            }
            Err(_) => {
                None
            }
        }

    }
}

impl<'font> Font<'font> {
    #[inline]
    pub fn inner(&self) -> &owned_ttf_parser::Face<'_> {
        match self {
            Self::Ref(f) => f,
            Self::Owned(f) => f.as_face_ref(),
        }
    }

    /// The "vertical metrics" for this font at a given scale. These metrics are
    /// shared by all of the glyphs in the font. See `VMetrics` for more detail.
    pub fn v_metrics(&self, scale: Scale) -> VMetrics {
        self.v_metrics_unscaled() * self.scale_for_pixel_height(scale.y)
    }

    /// Get the unscaled VMetrics for this font, shared by all glyphs.
    /// If you see that all the metrics are 0, it might be because the
    /// font you have loaded are not a real TrueType font, but rather
    /// a sfnt-housed font. To combat this, you can provide your own
    /// metrics for the font using `custom_v_metrics`.
    /// See `VMetrics` for more detail.
    pub fn v_metrics_unscaled(&self) -> VMetrics {
        let font = self.inner();

        VMetrics {
            ascent: font.ascender() as f32,
            descent: font.descender() as f32,
            line_gap: font.line_gap() as f32,
        }
    }

    /// Returns the units per EM square of this font
    pub fn units_per_em(&self) -> u16 {
        self.inner()
            .units_per_em()
    }

    /// The number of glyphs present in this font. Glyph identifiers for this
    /// font will always be in the range `0..self.glyph_count()`
    pub fn glyph_count(&self) -> usize {
        self.inner().number_of_glyphs() as _
    }

    /// Returns the corresponding glyph for a Unicode code point or a glyph id
    /// for this font.
    ///
    /// If `id` is a `GlyphId`, it must be valid for this font; otherwise, this
    /// function panics. `GlyphId`s should always be produced by looking up some
    /// other sort of designator (like a Unicode code point) in a font, and
    /// should only be used to index the font they were produced for.
    ///
    /// Note that code points without corresponding glyphs in this font map to
    /// the ".notdef" glyph, glyph 0.
    pub fn glyph<C: IntoGlyphId>(&self, id: C) -> Glyph<'font> {
        let gid = id.into_glyph_id(self);
        assert!((gid.0 as usize) < self.glyph_count());
        // font clone either a reference clone, or arc clone
        Glyph {
            font: self.clone(),
            id: gid,
        }
    }

    /// A convenience function.
    ///
    /// Returns an iterator that produces the glyphs corresponding to the code
    /// points or glyph ids produced by the given iterator `itr`.
    ///
    /// This is equivalent in behaviour to `itr.map(|c| font.glyph(c))`.
    pub fn glyphs_for<'a, I: Iterator>(&'a self, itr: I) -> GlyphIter<'a, 'font, I>
    where
        I::Item: IntoGlyphId,
    {
        GlyphIter { font: self, itr }
    }

    /// Returns additional kerning to apply as well as that given by HMetrics
    /// for a particular pair of glyphs.
    pub fn pair_kerning<A, B>(&self, scale: Scale, first: A, second: B) -> f32
    where
        A: IntoGlyphId,
        B: IntoGlyphId,
    {
        let first_id = first.into_glyph_id(self).into();
        let second_id = second.into_glyph_id(self).into();

        let factor = {
            let hscale = self.scale_for_pixel_height(scale.y);
            hscale * (scale.x / scale.y)
        };

        let kern = if let Some(kern) = self.inner().tables().kern {
            kern.subtables
                .into_iter()
                .filter(|st| st.horizontal && !st.variable)
                .find_map(|st| st.glyphs_kerning(first_id, second_id))
                .unwrap_or(0)
        } else {
            0
        };


        factor * f32::from(kern)
    }

    /// Computes a scale factor to produce a font whose "height" is 'pixels'
    /// tall. Height is measured as the distance from the highest ascender
    /// to the lowest descender; in other words, it's equivalent to calling
    /// GetFontVMetrics and computing:
    ///       scale = pixels / (ascent - descent)
    /// so if you prefer to measure height by the ascent only, use a similar
    /// calculation.
    pub fn scale_for_pixel_height(&self, height: f32) -> f32 {
        let inner = self.inner();
        let f_height = f32::from(inner.ascender()) - f32::from(inner.descender());

        height / f_height
    }


    pub fn glyph_id(&self, c: char) -> Option<GlyphId> {
        self.inner()
            .glyph_index(c)
            .map(|owned_ttf_parser::GlyphId(id)| GlyphId(id))
    }

    pub fn glyph_raster_image(&self, id: GlyphId, pixels_per_em: u16) -> Option<RasterGlyphImage> {
        self.inner().glyph_raster_image(
            owned_ttf_parser::GlyphId(id.0),
            pixels_per_em,
        )
    }

}
