// use crate::Scalar;
// use crate::text::{Font, FontSize};
// use crate::text::section::Section;
// use crate::widget::types::text_wrap::Wrap;
// use crate::text::text_span::TextSpan;
// use crate::widget::{GlobalState, Environment};
// use crate::text::text_style::TextStyle;
// use crate::draw::{Rect, Position, Dimension};
//
// #[derive(Debug, Clone)]
// pub struct Paragraph<GS> {
//     text: String,
//     pub max_width: Scalar,
//     span: TextSpan<GS>,
//     number_of_lines: u32,
//     hard_break: bool,
// }
//
// impl<GS: GlobalState> Paragraph<GS> {
//     /// Create a paragraph from a simple string.
//     pub fn new_simple(string: &str) -> Paragraph<GS> {
//         let span = TextSpan::new(string);
//         let total_width = span.max_width();
//
//         Paragraph {
//             text: string.to_string(),
//             max_width: total_width,
//             number_of_lines: 0,
//             span,
//             hard_break: false
//         }
//     }
//
//     pub fn update(&mut self, position: Position, bounding_size: Dimension, env: &mut Environment<GS>) -> Dimension {
//         let bounding_box = Rect {
//             position,
//             dimension: bounding_size
//         };
//         self.span.layout(bounding_box, env)
//     }
//
//
//
//     fn recalculate_lines(&mut self, bound_width: Scalar) {
//         let mut current_width = 0.0;
//
//         for section in &mut self.sections {
//             current_width += section.total_width();
//             section.set_line_number((current_width / bound_width) as u32);
//             self.number_of_lines = (current_width / bound_width) as u32;
//         }
//     }
//
//     /// Calculates the height of the paragraph based on the parameters and returns the new height
//     pub fn height(&mut self, font_size: FontSize, bound_width: Scalar) -> Scalar {
//         if self.max_width > bound_width {
//             self.recalculate_lines(bound_width);
//
//             let number_of_lines_in_paragraph = self.sections[self.sections.len() - 1].line_number();
//             let line_spacing = 1.0;
//
//             number_of_lines_in_paragraph as Scalar
//                 * font_size as Scalar
//                 + (number_of_lines_in_paragraph - 1) as Scalar
//                 * line_spacing
//         } else {
//             font_size as f64
//         }
//     }
//
//     pub fn max_width(&self) -> Scalar {
//         self.max_width
//     }
//
//     pub fn set_font_size(&mut self, font_size: FontSize) {
//         match self.span {
//             TextSpan::Text { style, .. } => {
//                 if let Some(mut style) = style {
//                     style.font_size = font_size
//                 }
//             }
//             TextSpan::List { .. } => {}
//             TextSpan::Widget(_) => {}
//         }
//     }
// }
