use carbide_core::color::Color;
use carbide_core::environment::Environment;
use carbide_core::text::{FontId, FontSize};
use carbide_core::text::FontStyle;
use carbide_core::text::FontWeight;
use carbide_core::text::TextDecoration;
use crate::font::Font;
use crate::text_context::TextContext;

/// The text style for a piece of text
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    /// A key to get the font family from the env.
    pub font_family: String,
    /// The size of the text
    pub font_size: FontSize,
    /// Whether the font should be italic or normal
    pub font_style: FontStyle,
    /// The weight of the font to show in
    pub font_weight: FontWeight,
    /// Underline, Overline, StrikeThrough
    pub text_decoration: TextDecoration,
    /// The primary color for the text
    pub color: Option<Color>,
}

impl TextStyle {
    pub fn get_font(&self, context: &TextContext) -> Font {
        let family = context.get_font_family(&self.font_family);
        let font_id = family.get_best_fit(self.font_weight, self.font_style);
        context.get_font(font_id)
    }

    pub fn get_font_id(&self, context: &TextContext) -> FontId {
        let family = context.get_font_family(&self.font_family);
        family.get_best_fit(self.font_weight, self.font_style)
    }
}
