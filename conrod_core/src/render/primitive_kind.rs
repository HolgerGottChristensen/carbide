use ::{Color, Point};
use color::Rgba;
use widget::triangles::{Triangle, ColoredPoint};
use ::{image, Rect};
use render::text::Text;
use ::{text, graph};

/// The unique kind for each primitive element in the Ui.
pub enum PrimitiveKind {

    /// A filled `Rectangle`.
    ///
    /// These are produced by the `Rectangle` and `BorderedRectangle` primitive widgets. A `Filled`
    /// `Rectangle` widget produces a single `Rectangle`. The `BorderedRectangle` produces two
    /// `Rectangle`s, the first for the outer border and the second for the inner on top.
    Rectangle {
        /// The fill colour for the rectangle.
        color: Color
    },

    /// A series of consecutive `Triangles` that are all the same color.
    TrianglesSingleColor {
        /// The color of all triangles.
        color: Rgba,
        /// An ordered slice of triangles.
        triangles: Vec<Triangle<Point>>
    },

    /// A series of consecutive `Triangles` with unique colors per vertex.
    ///
    /// This variant is produced by the general purpose `Triangles` primitive widget.
    TrianglesMultiColor {
        /// An ordered slice of multicolored triangles.
        triangles: Vec<Triangle<ColoredPoint>>
    },

    /// A single `Image`, produced by the primitive `Image` widget.
    Image {
        /// The unique identifier of the image that will be drawn.
        image_id: image::Id,
        /// When `Some`, colours the `Image`. When `None`, the `Image` uses its regular colours.
        color: Option<Color>,
        /// The area of the texture that will be drawn to the `Image`'s `Rect`.
        source_rect: Option<Rect>,
    },

    /// A single block of `Text`, produced by the primitive `Text` widget.
    Text {
        /// The colour of the `Text`.
        color: Color,
        /// All glyphs within the `Text` laid out in their correct positions in order from top-left
        /// to bottom right.
        text: Text,
        /// The unique identifier for the font, useful for the `glyph_cache.rect_for(id, glyph)`
        /// method when using the `conrod::text::GlyphCache` (rusttype's GPU `Cache`).
        font_id: text::font::Id,
    },

    /// An `Other` variant will be yielded for every non-primitive widget in the list.
    ///
    /// Most of the time, this variant can be ignored, however it is useful for users who need to
    /// render widgets in ways that cannot be covered by the other `PrimitiveKind` variants.
    ///
    /// For example, a `Shader` widget might be required for updating uniforms in user rendering
    /// code. In order to access the unique state of this widget, the user can check `Other`
    /// variants for a container whose `kind` field matches the unique kind of the `Shader` widget.
    /// They can then retrieve the unique state of the widget and cast it to its actual type using
    /// either of the `Container::state_and_style` or `Container::unique_widget_state` methods.
    Other(graph::Container),

}

