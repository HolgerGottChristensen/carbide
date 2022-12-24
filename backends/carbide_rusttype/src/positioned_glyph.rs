use std::fmt;
use crate::{Font, GlyphId, NearZero, OutlineBuilder, outliner, Point, Rect, Scale, vector};
use crate::scaled_glyph::ScaledGlyph;

/// A glyph augmented with positioning and scaling information. You can query
/// such a glyph for information that depends on the scale and position of the
/// glyph.
#[derive(Clone)]
pub struct PositionedGlyph<'font> {
    pub(crate) sg: ScaledGlyph<'font>,
    pub(crate) position: Point<f32>,
    pub(crate) bb: Option<Rect<i32>>,
}

impl<'font> PositionedGlyph<'font> {
    /// The glyph identifier for this glyph.
    pub fn id(&self) -> GlyphId {
        self.sg.id()
    }

    /// The font to which this glyph belongs.
    #[inline]
    pub fn font(&self) -> &Font<'font> {
        self.sg.font()
    }

    /// A reference to this glyph without positioning
    pub fn unpositioned(&self) -> &ScaledGlyph<'font> {
        &self.sg
    }

    /// Removes the positioning from this glyph
    pub fn into_unpositioned(self) -> ScaledGlyph<'font> {
        self.sg
    }

    /// The conservative pixel-boundary bounding box for this glyph. This is the
    /// smallest rectangle aligned to pixel boundaries that encloses the shape
    /// of this glyph at this position. Note that the origin of the glyph, at
    /// pixel-space coordinates (0, 0), is at the top left of the bounding box.
    pub fn pixel_bounding_box(&self) -> Option<Rect<i32>> {
        self.bb
    }

    pub fn scale(&self) -> Scale {
        self.sg.api_scale
    }

    pub fn position(&self) -> Point<f32> {
        self.position
    }

    /// Builds the outline of the glyph with the builder specified. Returns
    /// `false` when the outline is either malformed or empty.
    pub fn build_outline(&self, builder: &mut impl OutlineBuilder) -> bool {
        let bb = if let Some(bb) = self.bb.as_ref() {
            bb
        } else {
            return false;
        };

        let offset = vector(bb.min.x as f32, bb.min.y as f32);

        let mut outliner = outliner::OutlineTranslator::new(builder, self.position - offset);

        self.sg.build_outline(&mut outliner)
    }

    /// Rasterises this glyph. For each pixel in the rect given by
    /// `pixel_bounding_box()`, `o` is called:
    ///
    /// ```ignore
    /// o(x, y, v)
    /// ```
    ///
    /// where `x` and `y` are the coordinates of the pixel relative to the `min`
    /// coordinates of the bounding box, and `v` is the analytically calculated
    /// coverage of the pixel by the shape of the glyph. Calls to `o` proceed in
    /// horizontal scanline order, similar to this pseudo-code:
    ///
    /// ```ignore
    /// let bb = glyph.pixel_bounding_box();
    /// for y in 0..bb.height() {
    ///     for x in 0..bb.width() {
    ///         o(x, y, calc_coverage(&glyph, x, y));
    ///     }
    /// }
    /// ```
    pub fn draw<O: FnMut(u32, u32, f32)>(&self, o: O) {
        let bb = if let Some(bb) = self.bb.as_ref() {
            bb
        } else {
            return;
        };

        let width = (bb.max.x - bb.min.x) as u32;
        let height = (bb.max.y - bb.min.y) as u32;

        let mut outliner = crate::outliner::OutlineRasterizer::new(width as _, height as _);

        self.build_outline(&mut outliner);

        outliner.rasterizer.for_each_pixel_2d(o);
    }

    /// Resets positioning information and recalculates the pixel bounding box
    pub fn set_position(&mut self, p: Point<f32>) {
        let p_diff = p - self.position;
        if p_diff.x.fract().is_near_zero() && p_diff.y.fract().is_near_zero() {
            if let Some(bb) = self.bb.as_mut() {
                let rounded_diff = vector(p_diff.x.round() as i32, p_diff.y.round() as i32);
                bb.min = bb.min + rounded_diff;
                bb.max = bb.max + rounded_diff;
            }
        } else {
            self.bb = self.sg.pixel_bounds_at(p);
        }
        self.position = p;
    }
}

impl fmt::Debug for PositionedGlyph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PositionedGlyph")
            .field("id", &self.id().0)
            .field("scale", &self.scale())
            .field("position", &self.position)
            .finish()
    }
}
