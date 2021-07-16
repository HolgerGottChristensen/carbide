use rusttype::{GlyphId, point, PositionedGlyph, Scale};

use crate::{Point, Scalar};
use crate::draw::{Dimension, Position, Rect};
use crate::mesh::TextureAtlasIndex;
use crate::text::{Font, FontId, FontSize, InnerGlyph};
use crate::text::text_style::TextStyle;
use crate::widget::{Environment, GlobalState};

#[derive(Debug, Clone)]
pub struct Glyph {
    /// The id of a glyph if specific to a font. This glyph should only be
    /// used in the context of the font.
    id: GlyphId,
    /// The id of the font to use to render this glyph. This is important in mesh.rs
    font_id: FontId,
    bitmap_glyph: bool,
    /// The scale given when changing the glyph to a scaled glyph
    api_scale: Dimension,
    /// Scale calculated when changing to scaled glyph
    scale: Dimension,

    font_size: FontSize,

    position: Position,
    bb: Option<Rect>,

    /// The index of this glyph in the texture atlas.
    /// If this is None, this glyph is not queued.
    texture_index: Option<TextureAtlasIndex>,
}

impl Glyph {
    pub fn id(&self) -> GlyphId {
        self.id
    }

    pub fn font_size(&self) -> FontSize {
        self.font_size
    }

    pub fn font_id(&self) -> FontId {
        self.font_id
    }

    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn scale(&self) -> Dimension {
        self.api_scale
    }

    pub fn bb(&self) -> Option<Rect> {
        self.bb
    }

    pub fn set_texture_index(&mut self, index: TextureAtlasIndex) {
        self.texture_index = Some(index)
    }

    pub fn texture_index(&self) -> Option<TextureAtlasIndex> {
        self.texture_index
    }

    pub fn is_bitmap(&self) -> bool {
        self.bitmap_glyph
    }
}

impl From<(FontSize, FontId, PositionedGlyph<'_>, bool)> for Glyph {
    fn from((font_size, font_id, inner, is_bitmap): (FontSize, FontId, PositionedGlyph, bool)) -> Self {
        let scale = inner.scale();
        let scale_y = inner.font().scale_for_pixel_height(scale.y);
        let scale_x = scale_y * scale.x / scale.y;

        Glyph {
            id: inner.id(),
            font_id,
            bitmap_glyph: is_bitmap,
            api_scale: Dimension::new(inner.scale().x as f64, inner.scale().y as f64),
            scale: Dimension::new(scale_x as f64, scale_y as f64),
            font_size,
            position: Position::new(inner.position().x as f64, inner.position().y as f64),
            bb: inner.pixel_bounding_box().map(|bb| {
                let width = bb.max.x as f64 - bb.min.x as f64;
                let height = bb.max.y as f64 - bb.min.y as f64;

                Rect {
                    position: Position::new(bb.min.x as f64, bb.min.y as f64),
                    dimension: Dimension::new(width, height),
                }
            }),
            texture_index: None,
        }
    }
}