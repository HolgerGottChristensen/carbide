use rusttype::{GlyphId, point, PositionedGlyph, Scale};

use crate::{Point, Scalar};
use crate::draw::{Dimension, Position, Rect};
use crate::text::{Font, InnerGlyph};
use crate::text::text_style::TextStyle;
use crate::widget::{Environment, GlobalState};

#[derive(Debug, Clone)]
pub struct Glyph {
    /// The id of a glyph if specific to a font. This glyph should only be
    /// used in the context of the font. Todo: Consider storing the font id
    id: GlyphId,
    /// The scale given when changing the glyph to a scaled glyph
    api_scale: Dimension,
    /// Scale calculated when changing to scaled glyph
    scale: Dimension,

    position: Position,
    bb: Option<Rect>,
}

impl Glyph {
    pub fn convert_to_glyph(&self, font: &Font) -> PositionedGlyph<'static> {
        let glyph = font.get_inner().glyph(self.id);
        let scale = Scale { x: self.api_scale.width as f32, y: self.api_scale.height as f32 };
        let scaled = glyph.scaled(scale);
        scaled.positioned(point(self.position.x as f32, self.position.y as f32))
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
}

impl From<PositionedGlyph<'_>> for Glyph {
    fn from(inner: PositionedGlyph) -> Self {
        let scale = inner.scale();
        let scale_y = inner.font().scale_for_pixel_height(scale.y);
        let scale_x = scale_y * scale.x / scale.y;

        Glyph {
            id: inner.id(),
            api_scale: Dimension::new(inner.scale().x as f64, inner.scale().y as f64),
            scale: Dimension::new(scale_x as f64, scale_y as f64),
            position: Position::new(inner.position().x as f64, inner.position().y as f64),
            bb: inner.pixel_bounding_box().map(|bb| {
                Rect {
                    position: Position::new(bb.min.x as f64, bb.min.y as f64),
                    dimension: Dimension::new(bb.max.x as f64, bb.max.y as f64),
                }
            }),
        }
    }
}