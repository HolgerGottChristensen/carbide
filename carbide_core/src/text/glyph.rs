use carbide_core::mesh::AtlasEntry;
use carbide_rusttype::{GlyphId, point};
use carbide_rusttype::PositionedGlyph;

use crate::draw::{Position, Rect};
use crate::Scalar;
use crate::text::{FontId, FontSize};

pub const GLYPH_TOLERANCE: f64 = 0.25;

#[derive(Debug, Clone)]
pub struct Glyph {
    /// The id of the font to use to render this glyph. Use this ID to lookup the font when rendering.
    font_id: FontId,
    /// The fontsize of the current glyph.
    font_size: FontSize,
    /// Store if the glyph is made of a bitmap and should be rendered in color as is.
    bitmap_glyph: bool,
    /// The bounding box of the glyph. Is none if the glyph has no bounding box, such as a space char.
    bb: Option<Rect>,
    /// The index of this glyph in the texture atlas.
    /// If this is None, this glyph is not queued.
    atlas_entry: Option<AtlasEntry>,
    character: char,
    inner: PositionedGlyph<'static>,
}

impl Glyph {
    pub fn new(character: char, font_size: FontSize, font_id: FontId, inner: PositionedGlyph<'static>, is_bitmap: bool) -> Self {
        Glyph {
            font_id,
            bitmap_glyph: is_bitmap,
            font_size,
            bb: None,
            atlas_entry: None,
            character,
            inner
        }
    }

    /// The id of a glyph if specific to a font. This glyph should only be
    /// used in the context of the font.
    pub fn id(&self) -> GlyphId {
        self.inner.id()
    }

    pub fn character(&self) -> char {
        self.character
    }

    pub fn font_size(&self) -> FontSize {
        self.font_size
    }

    pub fn font_id(&self) -> FontId {
        self.font_id
    }

    pub fn position(&self) -> Position {
        self.inner.position().into()
    }

    pub fn width_of_glyph_from_origin(&self) -> Scalar {
        self.inner.unpositioned().h_metrics().left_side_bearing as Scalar
    }

    pub fn advance_width(&self) -> Scalar {
        self.inner.unpositioned().h_metrics().advance_width as Scalar
    }

    pub fn set_position(&mut self, mut position: Position) {
        position = position.tolerance(GLYPH_TOLERANCE);

        self.inner.set_position(point(position.x as f32, position.y as f32));

        self.bb = self.inner.pixel_bounding_box().map(Rect::from);
    }

    pub fn bb(&self) -> Option<Rect> {
        self.bb
    }

    pub fn with_scale_factor(mut self, scale_factor: Scalar) -> Self {
        self.bb = self.bb.map(|a| a / scale_factor);
        self
    }

    pub fn set_atlas_entry(&mut self, entry: AtlasEntry) {
        self.atlas_entry = Some(entry)
    }

    pub fn atlas_entry(&self) -> &Option<AtlasEntry> {
        &self.atlas_entry
    }

    pub fn is_bitmap(&self) -> bool {
        self.bitmap_glyph
    }
}
