use rusttype::Scale;

use crate::{FontSize, Rect};
use crate::position::{Align, Dimensions};
use crate::Scalar;
use crate::text;
use crate::text::Justify;
use crate::text::line::Info;

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
    pub(crate) base_line_offset: f32
}


impl Text {

    /// Produces a list of `PositionedGlyph`s which may be used to cache and render the text.
    ///
    /// `dpi_factor`, aka "dots per inch factor" is a multiplier representing the density of
    /// the display's pixels. The `Scale` of the font will be multiplied by this factor in order to
    /// ensure that each `PositionedGlyph`'s `pixel_bounding_box` is accurate and that the GPU
    /// cache receives glyphs of a size that will display correctly on displays regardless of DPI.
    ///
    /// Note that carbide does not require this factor when instantiating `Text` widgets and laying
    /// out text. This is because carbide positioning uses a "pixel-agnostic" `Scalar` value
    /// representing *perceived* distances for its positioning and layout, rather than pixel
    /// values. During rendering however, the pixel density must be known
    pub fn positioned_glyphs(self, scale_factor: f32) -> Vec<text::PositionedGlyph> {
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
            base_line_offset,
        } = self;

        //let rect = Rect::from_xy_dim([rect.x(),0.0], rect.dim());

        // Convert carbide coordinates to pixel coordinates.
        let trans_x = |x: Scalar| (x + window_dim[0] / 2.0 - rect.w() / 2.0) * scale_factor as Scalar;
        //let trans_y = |y: Scalar| ((y) + window_dim[1] / 2.0 - base_line_offset as f64) * dpi_factor as Scalar;
        let trans_y = |y: Scalar| ((y) + window_dim[1] / 2.0) * scale_factor as Scalar;

        // Produce the text layout iterators.
        let line_infos = line_infos.iter().cloned();
        let lines = line_infos.clone().map(|info| &text[info.byte_range()]);
        let line_rects = Self::calculate_line_rects(
            line_infos.collect::<Vec<_>>(),
            font_size,
            rect,
            justify,
            line_spacing,
            scale_factor as f64,
            base_line_offset as f64,
        );

        // Clear the existing glyphs and fill the buffer with glyphs for this Text.
        positioned_glyphs.clear();
        let scale = text::f32_pt_to_scale(font_size as f32 * scale_factor);
        for (line, line_rect) in lines.zip(line_rects.iter()) {
            let point = text::rt::Point { x: line_rect.x.start as f32, y: line_rect.y.start as f32 };
            positioned_glyphs.extend(font.layout(line, scale, point).map(|g| g.standalone()));
        }

        positioned_glyphs
    }

    fn calculate_line_rects(
        infos: Vec<Info>,
        font_size: u32,
        bounding_box: Rect,
        justify: Justify,
        line_spacing: f64,
        scale_factor: f64,
        baseline_offset: f64,
    ) -> Vec<Rect> {
        let mut current_advancement_y = bounding_box.y.start + baseline_offset;

        let font_size = font_size as f64;

        let rects = infos.iter().map(|info| {
            let width = info.width;
            let height = font_size;
            let dimension: Dimensions = [width, height];
            let x = match justify {
                Justify::Left => {
                    bounding_box.x.start
                }
                Justify::Center => {
                    (bounding_box.x.start + bounding_box.x.end) / 2.0 - width / 2.0
                }
                Justify::Right => {
                    bounding_box.x.end - width
                }
            };

            current_advancement_y += height + line_spacing;

            let point = [x * scale_factor, current_advancement_y * scale_factor];

            Rect::new(point, dimension)
        });

        rects.collect::<Vec<_>>()
    }
}


