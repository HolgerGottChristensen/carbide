use carbide::draw::Scalar;
use carbide_core::color::Color;
use carbide_core::text::FontSize;
use carbide_core::text::FontStyle;
use carbide_core::text::FontWeight;
use carbide_core::text::TextDecoration;
use crate::widget::Wrap;

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