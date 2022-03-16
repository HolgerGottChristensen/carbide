use cgmath::Matrix4;

use crate::Color;
use crate::color::Rgba;
use crate::draw::{Position, Rect};
use crate::draw::draw_gradient::DrawGradient;
use crate::draw::shape::triangle::Triangle;
use crate::image_map;
use crate::layout::BasicLayouter;
use crate::text::Glyph;
use crate::widget::{ColoredPoint, Gradient};

/// The unique kind for each primitive element in the Ui.
pub enum PrimitiveKind {
    /// Start a clip for the rectangle given by the primitive
    Clip,
    /// Remove a clip
    UnClip,

    Stencil(Vec<Triangle<Position>>),
    DeStencil,

    /// This is a filter and can take any 2d filter
    Filter(u32),

    /// A part 1 should always be followed directly by a part 2. This is more performant for
    /// filters that are seperable such as gaussian and box blur.
    FilterSplitPt1(u32),
    FilterSplitPt2(u32),

    Transform(Matrix4<f32>, BasicLayouter),
    DeTransform,

    /// A filled `Rectangle`.
    ///
    /// These are produced by the `Rectangle` and `BorderedRectangle` primitive widgets. A `Filled`
    /// `Rectangle` widget produces a single `Rectangle`. The `BorderedRectangle` produces two
    /// `Rectangle`s, the first for the outer border and the second for the inner on top.
    RectanglePrim {
        /// The fill colour for the rectangle.
        color: Color,
    },

    /// A series of consecutive `Triangles` that are all the same color.
    TrianglesSingleColor {
        /// The color of all triangles.
        color: Rgba,
        //Todo why is this not Color
        /// An ordered slice of triangles.
        triangles: Vec<Triangle<Position>>,
    },

    Gradient(Vec<Triangle<Position>>, DrawGradient),

    /// A series of consecutive `Triangles` with unique colors per vertex.
    ///
    /// This variant is produced by the general purpose `Triangles` primitive widget.
    TrianglesMultiColor {
        /// An ordered slice of multicolored triangles.
        triangles: Vec<Triangle<ColoredPoint>>,
    },

    /// A single `Image`, produced by the primitive `Image` widget.
    Image {
        /// The unique identifier of the image that will be drawn.
        image_id: image_map::Id,
        /// When `Some`, colours the `Image`. When `None`, the `Image` uses its regular colours.
        color: Option<Color>,
        /// The area of the texture that will be drawn to the `Image`'s `Rect`.
        source_rect: Option<Rect>,
        /// The mode of the image to draw, most commonly MODE_ICON or MODE_IMAGE
        mode: u32,
    },

    /// A single block of `Text`, produced by the primitive `Text` widget.
    Text {
        /// The colour of the `Text`.
        color: Color,
        /// All glyphs within the `Text` laid out in their correct positions in order from top-left
        /// to bottom right.
        text: Vec<Glyph>,
    },
}
