// use crate::environment::environment::Environment;
// use crate::position::Dimensions;
// use crate::OldRect;
// use crate::Scalar;
// use crate::text::{FontId, FontSize, Font};
// use crate::text_old;
// use crate::text_old::line::Info;
// use crate::widget::GlobalState;
// use crate::widget::types::justify;
// use crate::widget::types::justify::Justify;
// use rusttype::{PositionedGlyph, Glyph, point};
// use crate::text::Text as InternalText;
//
// /// A type used for producing a `PositionedGlyph` iterator.
// ///
// /// We produce this type rather than the `&[PositionedGlyph]`s directly so that we can properly
// /// handle "HiDPI" scales when caching glyphs.
// pub struct Text {
//     pub(crate) internal_text: InternalText<GS>,
//     pub(crate) font_size: FontSize,
//     pub(crate) rect: OldRect,
//     pub(crate) justify: justify::Justify,
//     pub(crate) line_spacing: Scalar,
//     pub(crate) base_line_offset: f32,
// }
//
// impl Text {
//
//     /// Produces a list of `PositionedGlyph`s which may be used to cache and render the text.
//     ///
//     /// `dpi_factor`, aka "dots per inch factor" is a multiplier representing the density of
//     /// the display's pixels. The `Scale` of the font will be multiplied by this factor in order to
//     /// ensure that each `PositionedGlyph`'s `pixel_bounding_box` is accurate and that the GPU
//     /// cache receives glyphs of a size that will display correctly on displays regardless of DPI.
//     ///
//     /// Note that carbide does not require this factor when instantiating `Text` widgets and laying
//     /// out text. This is because carbide positioning uses a "pixel-agnostic" `Scalar` value
//     /// representing *perceived* distances for its positioning and layout, rather than pixel
//     /// values. During rendering however, the pixel density must be known
//     pub fn positioned_glyphs<GS: GlobalState>(&self, env: &Environment<GS>, scale_factor: f32) -> Vec<text_old::PositionedGlyph> {
//         let font = self.internal_text.font(env);
//
//         let text = self.internal_text.text();
//         let scale = text_old::f32_pt_to_scale(self.font_size as f32 * scale_factor);
//
//         let mut x = self.rect.x.start;
//         let mut y = self.rect.y.start + self.internal_text.line_height(self.font_size, env);
//
//         println!("x: {}, y: {}", x, y);
//
//         let positioned: Vec<PositionedGlyph> = text.chars().map(|c| {
//             let glyph = font.get_inner().glyph(c);
//             let scaled_glyph = glyph.scaled(scale);
//             scaled_glyph.positioned(point(x as f32 * scale_factor, y as f32 * scale_factor))
//         }).map(|g| g.standalone()).collect();
//
//         positioned
//     }
//
//     fn calculate_line_rects(
//         infos: Vec<Info>,
//         font_size: u32,
//         bounding_box: OldRect,
//         justify: Justify,
//         line_spacing: f64,
//         scale_factor: f64,
//         baseline_offset: f64,
//     ) -> Vec<OldRect> {
//         let mut current_advancement_y = bounding_box.y.start + baseline_offset;
//
//         let font_size = font_size as f64;
//
//         let rects = infos.iter().map(|info| {
//             let width = info.width;
//             let height = font_size;
//             let dimension: Dimensions = [width, height];
//             let x = match justify {
//                 Justify::Left => {
//                     bounding_box.x.start
//                 }
//                 Justify::Center => {
//                     (bounding_box.x.start + bounding_box.x.end) / 2.0 - width / 2.0
//                 }
//                 Justify::Right => {
//                     bounding_box.x.end - width
//                 }
//             };
//
//             current_advancement_y += height + line_spacing;
//
//             let point = [x * scale_factor, current_advancement_y * scale_factor];
//
//             OldRect::new(point, dimension)
//         });
//
//         rects.collect::<Vec<_>>()
//     }
// }
//
//
