use ::{text};
use ::{FontSize, Rect};
use position::{Align, Dimensions};
use Scalar;

/// A type used for producing a `PositionedGlyph` iterator.
///
/// We produce this type rather than the `&[PositionedGlyph]`s directly so that we can properly
/// handle "HiDPI" scales when caching glyphs.
pub struct Text {
    pub(crate) positioned_glyphs: Vec<text::PositionedGlyph>,
    pub(crate) window_dim: Dimensions,
    pub(crate) text: String,
    pub(crate) line_infos: Vec<text::line::Info>,
    pub(crate) font: text::Font,
    pub(crate) font_size: FontSize,
    pub(crate) rect: Rect,
    pub(crate) justify: text::Justify,
    pub(crate) y_align: Align,
    pub(crate) line_spacing: Scalar,
}


impl Text {

    /// Produces a list of `PositionedGlyph`s which may be used to cache and render the text.
    ///
    /// `dpi_factor`, aka "dots per inch factor" is a multiplier representing the density of
    /// the display's pixels. The `Scale` of the font will be multiplied by this factor in order to
    /// ensure that each `PositionedGlyph`'s `pixel_bounding_box` is accurate and that the GPU
    /// cache receives glyphs of a size that will display correctly on displays regardless of DPI.
    ///
    /// Note that conrod does not require this factor when instantiating `Text` widgets and laying
    /// out text. This is because conrod positioning uses a "pixel-agnostic" `Scalar` value
    /// representing *perceived* distances for its positioning and layout, rather than pixel
    /// values. During rendering however, the pixel density must be known
    pub fn positioned_glyphs(self, dpi_factor: f32) -> Vec<text::PositionedGlyph> {
        let Text {
            mut positioned_glyphs,
            window_dim,
            text,
            line_infos,
            font,
            font_size,
            rect,
            justify,
            y_align,
            line_spacing,
        } = self;

        // Convert conrod coordinates to pixel coordinates.
        let trans_x = |x: Scalar| (x + window_dim[0] / 2.0) * dpi_factor as Scalar;
        let trans_y = |y: Scalar| ((-y) + window_dim[1] / 2.0) * dpi_factor as Scalar;

        // Produce the text layout iterators.
        let line_infos = line_infos.iter().cloned();
        let lines = line_infos.clone().map(|info| &text[info.byte_range()]);
        let line_rects = text::line::rects(line_infos, font_size, rect,
                                           justify, y_align, line_spacing);

        // Clear the existing glyphs and fill the buffer with glyphs for this Text.
        positioned_glyphs.clear();
        let scale = text::f32_pt_to_scale(font_size as f32 * dpi_factor);
        for (line, line_rect) in lines.zip(line_rects) {
            let (x, y) = (trans_x(line_rect.left()) as f32, trans_y(line_rect.bottom()) as f32);
            let point = text::rt::Point { x, y };
            positioned_glyphs.extend(font.layout(line, scale, point).map(|g| g.standalone()));
        }

        positioned_glyphs
    }

}


