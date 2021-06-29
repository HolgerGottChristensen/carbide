use crate::Color;
use crate::environment::environment::Environment;
use crate::text::{Font, FontId, FontSize};
use crate::text::font_family::FontFamily;
use crate::text::font_style::FontStyle;
use crate::text::font_weight::FontWeight;
use crate::text::text_decoration::TextDecoration;
use crate::widget::GlobalState;
use crate::widget::types::justify::Justify;
use crate::widget::types::text_wrap::Wrap;

/// The text style for a piece of text
#[derive(Clone, Debug)]
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
    pub fn get_font<'a, GS: GlobalState>(&'a self, env: &'a mut Environment<GS>) -> &'a mut Font {
        let family = env.get_font_family(&self.font_family);
        let font_id = family.get_best_fit(self.font_weight, self.font_style);
        env.get_font_mut(font_id)
    }

    pub fn get_font_id<'a, GS: GlobalState>(&'a self, env: &'a mut Environment<GS>) -> FontId {
        let family = env.get_font_family(&self.font_family);
        family.get_best_fit(self.font_weight, self.font_style)
    }
}