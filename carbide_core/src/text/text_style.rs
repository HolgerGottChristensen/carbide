use crate::Color;
use crate::environment::environment::Environment;
use crate::text::{Font, FontSize};
use crate::text::font_family::FontFamily;
use crate::text::font_style::FontStyle;
use crate::text::font_weight::FontWeight;
use crate::text::text_decoration::TextDecoration;
use crate::widget::GlobalState;
use crate::widget::types::justify::Justify;
use crate::widget::types::text_wrap::Wrap;

/// The text style for a piece of text
#[derive(Copy, Clone, Debug)]
pub struct TextStyle {
    /// The family of fonts to display the piece of text in
    pub font_family: FontFamily,
    /// The size of the text
    pub font_size: FontSize,
    /// Whether the font should be italic or normal
    pub font_style: FontStyle,
    /// The weight of the font to show in
    pub font_weight: FontWeight,
    /// Underline, Overline, Strikethrough
    pub text_decoration: TextDecoration,
    /// The primary color for the text
    pub color: Color,
}

impl TextStyle {
    pub fn get_font<'a, GS: GlobalState>(&'a self, env: &'a mut Environment<GS>) -> &'a mut Font {
        env.get_font_mut(1)
    }
}