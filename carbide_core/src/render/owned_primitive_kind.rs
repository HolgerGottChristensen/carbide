use crate::{Color, OldRect};
use crate::color::Rgba;
use crate::image_map::Id;
use crate::render::owned_text::OwnedText;
use crate::text::FontId;

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
        source_rect: Option<OldRect>,
    },
    Text {
        color: Color,
        font_id: FontId,
        text: OwnedText,
    },
}

