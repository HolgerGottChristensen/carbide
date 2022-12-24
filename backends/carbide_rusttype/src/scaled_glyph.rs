use std::fmt;
use crate::glyph::Glyph;
use crate::{Font, GlyphId, HMetrics, OutlineBuilder, outliner, Point, point, Rect, Scale, Vector, vector};
use crate::positioned_glyph::PositionedGlyph;

/// A glyph augmented with scaling information. You can query such a glyph for
/// information that depends on the scale of the glyph.
#[derive(Clone)]
pub struct ScaledGlyph<'font> {
    pub(crate) g: Glyph<'font>,
    pub(crate) api_scale: Scale,
    pub(crate) scale: Vector<f32>,
}

impl<'font> ScaledGlyph<'font> {
    /// The glyph identifier for this glyph.
    pub fn id(&self) -> GlyphId {
        self.g.id()
    }

    /// The font to which this glyph belongs.
    #[inline]
    pub fn font(&self) -> &Font<'font> {
        self.g.font()
    }

    /// A reference to this glyph without the scaling
    pub fn into_unscaled(self) -> Glyph<'font> {
        self.g
    }

    /// Removes the scaling from this glyph
    pub fn unscaled(&self) -> &Glyph<'font> {
        &self.g
    }

    /// Builds the outline of the glyph with the builder specified. Returns
    /// `false` when the outline is either malformed or empty.
    pub fn build_outline(&self, builder: &mut impl OutlineBuilder) -> bool {
        let mut outliner =
            outliner::OutlineScaler::new(builder, vector(self.scale.x, -self.scale.y));

        self.font()
            .inner()
            .outline_glyph(self.id().into(), &mut outliner)
            .is_some()
    }

    /// Augments this glyph with positioning information, making methods that
    /// depend on the position of the glyph available.
    pub fn positioned(self, p: Point<f32>) -> PositionedGlyph<'font> {
        let bb = self.pixel_bounds_at(p);
        PositionedGlyph {
            sg: self,
            position: p,
            bb,
        }
    }

    pub fn scale(&self) -> Scale {
        self.api_scale
    }

    /// Retrieves the "horizontal metrics" of this glyph. See `HMetrics` for
    /// more detail.
    pub fn h_metrics(&self) -> HMetrics {
        let inner = self.font().inner();
        let id = self.id().into();

        let advance = inner.glyph_hor_advance(id).unwrap();
        let left_side_bearing = inner.glyph_hor_side_bearing(id).unwrap();

        HMetrics {
            advance_width: advance as f32 * self.scale.x,
            left_side_bearing: left_side_bearing as f32 * self.scale.x,
        }
    }

    /// The bounding box of the shape of this glyph, not to be confused with
    /// `pixel_bounding_box`, the conservative pixel-boundary bounding box. The
    /// coordinates are relative to the glyph's origin.
    pub fn exact_bounding_box(&self) -> Option<Rect<f32>> {
        let owned_ttf_parser::Rect {
            x_min,
            y_min,
            x_max,
            y_max,
        } = self.font().inner().glyph_bounding_box(self.id().into())?;

        Some(Rect {
            min: point(x_min as f32 * self.scale.x, -y_max as f32 * self.scale.y),
            max: point(x_max as f32 * self.scale.x, -y_min as f32 * self.scale.y),
        })
    }

    fn glyph_bitmap_box_subpixel(
        &self,
        font: &Font<'font>,
        shift_x: f32,
        shift_y: f32,
    ) -> Option<Rect<i32>> {
        let owned_ttf_parser::Rect {
            x_min,
            y_min,
            x_max,
            y_max,
        } = font.inner().glyph_bounding_box(self.id().into())?;

        Some(Rect {
            min: point(
                (x_min as f32 * self.scale.x + shift_x).floor() as i32,
                (-y_max as f32 * self.scale.y + shift_y).floor() as i32,
            ),
            max: point(
                (x_max as f32 * self.scale.x + shift_x).ceil() as i32,
                (-y_min as f32 * self.scale.y + shift_y).ceil() as i32,
            ),
        })
    }

    #[inline]
    pub(crate) fn pixel_bounds_at(&self, p: Point<f32>) -> Option<Rect<i32>> {
        // Use subpixel fraction in floor/ceil rounding to eliminate rounding error
        // from identical subpixel positions
        let (x_trunc, x_fract) = (p.x.trunc() as i32, p.x.fract());
        let (y_trunc, y_fract) = (p.y.trunc() as i32, p.y.fract());

        let Rect { min, max } = self.glyph_bitmap_box_subpixel(self.font(), x_fract, y_fract)?;
        Some(Rect {
            min: point(x_trunc + min.x, y_trunc + min.y),
            max: point(x_trunc + max.x, y_trunc + max.y),
        })
    }
}

impl fmt::Debug for ScaledGlyph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScaledGlyph")
            .field("id", &self.id().0)
            .field("scale", &self.api_scale)
            .finish()
    }
}
