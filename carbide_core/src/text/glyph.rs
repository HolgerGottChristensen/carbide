use carbide_core::mesh::AtlasEntry;
use rusttype::{point, GlyphId, PositionedGlyph};

use crate::draw::{Dimension, Position, Rect};
use crate::text::{FontId, FontSize};
use crate::Scalar;

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
    atlas_entry: Option<AtlasEntry>,

    /// This bb has been scaled to the correct size.
    inner_glyph_bb: Option<rusttype::Rect<f32>>,
    width_of_glyph_from_origin: Scalar,
    advance_width: Scalar,

    character: char,
}

impl Glyph {
    pub fn new(character: char, font_size: FontSize, font_id: FontId, inner: PositionedGlyph, is_bitmap: bool) -> Self {
        let scale = inner.scale();
        let scale_y = inner.font().scale_for_pixel_height(scale.y);
        let scale_x = scale_y * scale.x / scale.y;

        let glyph_id = inner.id();
        let inner_glyph_bb = inner
            .font()
            .inner()
            .glyph_bounding_box(glyph_id.into())
            .map(|ttf_bb| rusttype::Rect {
                min: point(
                    ttf_bb.x_min as f32 * scale_x,
                    -ttf_bb.y_max as f32 * scale_y,
                ),
                max: point(
                    ttf_bb.x_max as f32 * scale_x,
                    -ttf_bb.y_min as f32 * scale_y,
                ),
            });

        Glyph {
            id: glyph_id,
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
            atlas_entry: None,
            inner_glyph_bb,
            width_of_glyph_from_origin: inner.unpositioned().h_metrics().left_side_bearing as f64,
            advance_width: inner.unpositioned().h_metrics().advance_width as f64,
            character
        }
    }

    pub fn id(&self) -> GlyphId {
        self.id
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
        self.position
    }

    pub fn width_of_glyph_from_origin(&self) -> Scalar {
        self.width_of_glyph_from_origin
    }

    pub fn advance_width(&self) -> Scalar {
        self.advance_width
    }

    pub fn set_position(&mut self, position: Position) {
        /*let difference = position - self.position;
        if difference.fraction().is_near_zero() {
            if let Some(mut bb) = self.bb {
                let rounded_difference = difference.rounded();
                bb = bb + rounded_difference;
            }
        } else {*/
        self.bb = self.recalculate_bb(position);
        //}
        self.position = position;
    }

    fn recalculate_bb(&self, position: Position) -> Option<Rect> {
        let fraction_of_position = position.fraction();
        let truncated = position.truncated();
        let translated_bb = self.inner_glyph_bb.map(|bb| rusttype::Rect {
            min: point(
                (bb.min.x as f64 + fraction_of_position.x).floor() + truncated.x,
                (bb.min.y as f64 + fraction_of_position.y).floor() + truncated.y,
            ),
            max: point(
                (bb.max.x as f64 + fraction_of_position.x).ceil() + truncated.x,
                (bb.max.y as f64 + fraction_of_position.y).ceil() + truncated.y,
            ),
        });

        translated_bb.map(|rect| {
            let width = rect.max.x - rect.min.x;
            let height = rect.max.y - rect.min.y;
            Rect {
                position: Position::new(rect.min.x, rect.min.y),
                dimension: Dimension::new(width, height),
            }
        })
    }

    pub fn api_scale(&self) -> Dimension {
        self.api_scale
    }

    pub fn scale(&self) -> Dimension {
        self.scale
    }

    pub fn bb(&self) -> Option<Rect> {
        self.bb
    }

    pub fn set_texture_index(&mut self, index: AtlasEntry) {
        self.atlas_entry = Some(index)
    }

    pub fn atlas_entry(&self) -> &Option<AtlasEntry> {
        &self.atlas_entry
    }

    pub fn is_bitmap(&self) -> bool {
        self.bitmap_glyph
    }

    // pub fn l_r_b_t(&self) -> Option<Rect> {
    //
    // }
}
