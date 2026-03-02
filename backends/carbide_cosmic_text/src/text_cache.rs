use cosmic_text::Buffer;
use carbide_core::color::Color;
use carbide_core::draw::{Dimension, Scalar};
use carbide_core::text::{FontSize, FontStyle, FontWeight, TextDecoration, TextStyle};
use carbide_core::text::text_wrap::Wrap;

pub struct TextEntry {
    pub buffer: Buffer,
    pub dimension: Dimension,
    pub atlas_enqueued: bool,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct TextKey {
    pub text: String,
    pub style: CachableTextStyle,
    pub width: u32,
    pub scale_factor: u64,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CachableTextStyle {
    /// Name of the font to use
    pub family: String,
    /// The size of the text
    pub font_size: FontSize,
    pub line_height: u64,
    /// Whether the font should be italic or normal
    pub font_style: FontStyle,
    /// The weight of the font to show in
    pub font_weight: FontWeight,
    /// Underline, Overline, StrikeThrough
    pub text_decoration: TextDecoration,
    /// The primary color for the text
    pub color: Option<(u32, u32, u32, u32)>,
    pub wrap: Wrap,
}