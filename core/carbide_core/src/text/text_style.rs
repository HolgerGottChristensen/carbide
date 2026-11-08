use crate::draw::Scalar;
use crate::color::Color;
use crate::text::FontSize;
use crate::text::FontStyle;
use crate::text::FontWeight;
use crate::text::TextDecoration;
use crate::text::text_wrap::Wrap;

/// The text style for a piece of text
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    /// Name of the font to use
    pub family: String,
    /// The size of the text
    pub font_size: FontSize,
    pub line_height: Scalar,
    /// Whether the font should be italic or normal
    pub font_style: FontStyle,
    /// The weight of the font to show in
    pub font_weight: FontWeight,
    /// Underline, Overline, StrikeThrough
    pub text_decoration: TextDecoration,
    /// The primary color for the text
    pub color: Option<Color>,
    pub wrap: Wrap,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            family: "Noto Sans".to_string(),
            font_size: 13,
            line_height: 1.2,
            font_style: FontStyle::Normal,
            font_weight: FontWeight::Normal,
            text_decoration: TextDecoration::None,
            color: None,
            wrap: Wrap::None,
        }
    }
}