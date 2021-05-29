use crate::{Rect, Scalar};
use crate::position::Dimensions;
use crate::text::FontSize;
use crate::text_old::Font;
use crate::widget::types::justify::Justify;

#[derive(Clone)]
pub struct OwnedText {
    pub str_byte_range: std::ops::Range<usize>,
    pub line_infos_range: std::ops::Range<usize>,
    pub window_dim: Dimensions,
    pub font: Font,
    pub font_size: FontSize,
    pub rect: Rect,
    pub justify: Justify,
    pub line_spacing: Scalar,
    pub base_line_offset: f32,
}

