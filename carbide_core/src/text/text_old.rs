// use crate::{Scalar, Point};
// use crate::text::{Font, FontId, FontSize};
// use crate::text::paragraph::Paragraph;
// use crate::widget::{Environment, GlobalState};
// use crate::widget::types::text_wrap::Wrap;
// use crate::text_old::{pt_to_px, f32_pt_to_px, PositionedGlyph};
// use crate::draw::Rect;
//
// #[derive(Debug, Clone)]
// pub struct Text<GS> {
//     text: String,
//     max_width: Scalar,
//     paragraphs: Vec<Paragraph<GS>>,
//     bounding_box: Rect,
//     latest_height: Scalar,
//     line_height: Scalar,
//     wrap_mode: Wrap,
//     recalculate_cache: bool,
//     rich_text: bool,
// }
//
// impl<GS> Text<GS> {
//
//     pub fn new() -> Text<GS> {
//         Text {
//             text: "".to_string(),
//             max_width: 0.0,
//             paragraphs: vec![],
//             latest_height: 0.0,
//             line_height: 0.0,
//             wrap_mode: Wrap::Character,
//             recalculate_cache: true,
//             bounding_box: Default::default(),
//             rich_text: false
//         }
//     }
//
//     pub fn set_text(&mut self, text: &str) {
//         self.text = text.to_string();
//         self.recalculate_cache = true;
//     }
//
//     pub fn set_font_size(&mut self, font_size: FontSize) {
//         self.paragraphs.iter_mut().for_each(|paragraph| {
//             paragraph.set_font_size(font_size)
//         });
//         self.recalculate_cache = true;
//     }
//
//     pub fn update<GS: GlobalState>(&mut self, bounding_box: Rect, env: &mut Environment<GS>) {
//
//         self.bounding_box.position = bounding_box.position;
//
//         if self.recalculate_cache || bounding_box.dimension != self.bounding_box.dimension {
//             self.bounding_box.dimension = bounding_box.dimension;
//             let mut position = self.bounding_box.position;
//             let mut dimension = self.bounding_box.dimension;
//
//             for paragraph in &mut self.paragraphs {
//                 let paragraph_size = paragraph.update(position, dimension, env);
//                 position += paragraph_size;
//                 dimension -= paragraph_size;
//             }
//
//             self.recalculate_cache = false;
//         }
//     }
//
//     pub fn recalculate_paragraphs(&mut self, font: &mut Font) {
//         let paragraphs: Vec<String> = self.text.replace("\r\n", "\n").split('\n').map(|c| c.to_string()).collect();
//
//         self.paragraphs = paragraphs.iter()
//             .map(|paragraph| {
//                 Paragraph::new_simple(&paragraph, font, self.font_size, self.wrap_mode)
//             }).collect();
//
//         let mut max = 0.0;
//
//         for i in self.paragraphs.iter().map(|paragraph| paragraph.max_width()) {
//             if i > max {
//                 max = i
//             }
//         }
//
//         self.max_width = max;
//     }
//
//     pub fn max_width(&self) -> Scalar {
//         let mut max = 0.0;
//         for paragraph in self.paragraphs {
//             if paragraph.max_width > max {
//                 max = paragraph.max_width;
//             }
//         }
//         max
//     }
//
//     pub fn height(&mut self, width_bound: Scalar) -> Scalar {
//         if width_bound == self.latest_available_width {
//             return self.latest_height;
//         }
//
//         self.latest_height = self.paragraphs.iter_mut()
//             .map(|paragraph| {
//                 paragraph.height(self.font_size, width_bound)
//             }).sum();
//         self.latest_available_width = width_bound;
//
//         self.latest_height
//     }
//
//     pub fn text(&self) -> &str {
//         &self.text
//     }
//
//     pub fn line_height<GS: GlobalState>(&self, env: &Environment<GS>) -> Scalar {
//         let scale = rusttype::Scale::uniform(pt_to_px(self.font_size));
//         let v_metrics = env.get_font(self.font_id).get_inner().v_metrics(scale);
//         (v_metrics.ascent + v_metrics.descent) as f64
//     }
//
//     pub fn font<'a, GS: GlobalState>(&'a self, env: &'a Environment<GS>) -> &'a Font {
//         env.get_font(self.font_id)
//     }
//
//     pub fn get_positioned_glyphs(&self, start: Point, scale_factor: Scalar) -> Vec<PositionedGlyph> {
//
//     }
// }
