use crate::{Color, Rect};
use crate::color::Rgba;
use crate::image::Id;
use crate::render::owned_text::OwnedText;
use crate::text::font;

#[derive(Clone)]
pub enum OwnedPrimitiveKind {
    Rectangle {
        color: Color,
    },
    TrianglesSingleColor {
        color: Rgba,
        triangle_range: std::ops::Range<usize>,
    },
    TrianglesMultiColor {
        triangle_range: std::ops::Range<usize>,
    },
    Image {
        image_id: Id,
        color: Option<Color>,
        source_rect: Option<Rect>,
    },
    Text {
        color: Color,
        font_id: font::Id,
        text: OwnedText,
    },
}

