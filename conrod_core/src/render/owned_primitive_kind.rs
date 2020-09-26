use ::{Color, Rect};
use color::Rgba;
use image::Id;
use text::font;
use render::owned_text::OwnedText;

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

