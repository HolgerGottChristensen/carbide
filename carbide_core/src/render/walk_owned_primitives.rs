use crate::{Point, text};
use crate::draw::shape::triangle::Triangle;
use crate::render::owned_primitive::OwnedPrimitive;
use crate::render::owned_primitive_kind::OwnedPrimitiveKind;
use crate::render::owned_text::OwnedText;
use crate::render::primitive::Primitive;
use crate::render::primitive_kind::PrimitiveKind;
use crate::render::primitive_walker::PrimitiveWalker;
use crate::render::text::Text;
use crate::widget::primitive::shape::triangles::ColoredPoint;

/// An iterator-like type for yielding `Primitive`s from an `OwnedPrimitives`.
pub struct WalkOwnedPrimitives<'a> {
    pub(crate) primitives: std::slice::Iter<'a, OwnedPrimitive>,
    pub(crate) triangles_single_color: &'a [Triangle<Point>],
    pub(crate) triangles_multi_color: &'a [Triangle<ColoredPoint>],
    pub(crate) line_infos: &'a [text::line::Info],
    pub(crate) texts_str: &'a str,
    pub(crate) positioned_glyphs: Vec<text::PositionedGlyph>,
}

impl<'a> WalkOwnedPrimitives<'a> {

    /// Yield the next `Primitive` in order or rendering depth, bottom to top.
    pub fn next(&mut self) -> Option<Primitive> {
        let WalkOwnedPrimitives {
            ref mut primitives,
            ref mut positioned_glyphs,
            triangles_single_color,
            triangles_multi_color,
            line_infos,
            texts_str,
        } = *self;

        primitives.next().map(move |&OwnedPrimitive { rect, ref kind }| {
            let new = |kind| Primitive {
                rect,
                kind,
            };

            match *kind {

                OwnedPrimitiveKind::Rectangle { color } => {
                    let kind = PrimitiveKind::Rectangle { color: color };
                    new(kind)
                },

                OwnedPrimitiveKind::TrianglesSingleColor { color, ref triangle_range } => {
                    let kind = PrimitiveKind::TrianglesSingleColor {
                        color,
                        triangles: triangles_single_color[triangle_range.clone()].to_owned(),
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::TrianglesMultiColor { ref triangle_range } => {
                    let kind = PrimitiveKind::TrianglesMultiColor {
                        triangles: triangles_multi_color[triangle_range.clone()].to_vec(),
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::Text { color, font_id, ref text } => {
                    let OwnedText {
                        ref str_byte_range,
                        ref line_infos_range,
                        ref font,
                        window_dim,
                        font_size,
                        rect,
                        justify,
                        y_align,
                        line_spacing,
                        base_line_offset,
                    } = *text;

                    let text_str = &texts_str[str_byte_range.clone()];
                    let line_infos = &line_infos[line_infos_range.clone()];

                    let text = Text {
                        positioned_glyphs: (*positioned_glyphs).clone(),
                        window_dim,
                        text: text_str.clone().parse().unwrap(),
                        line_infos: line_infos.to_vec(),
                        font: font.clone(),
                        font_size,
                        rect,
                        justify,
                        y_align,
                        line_spacing,
                        base_line_offset,
                    };

                    let kind = PrimitiveKind::Text {
                        color,
                        font_id,
                        text,
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::Image { image_id, color, source_rect } => {
                    let kind = PrimitiveKind::Image {
                        image_id,
                        color,
                        source_rect,
                    };
                    new(kind)
                },
            }
        })
    }

}


impl<'a> PrimitiveWalker for WalkOwnedPrimitives<'a> {
    fn next_primitive(&mut self) -> Option<Primitive> {
        self.next()
    }
}





