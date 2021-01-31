use crate::FontSize;
use crate::{Rect, Scalar};
use crate::position::{Align, Dimensions};
use crate::text::{Font, Justify};

#[derive(Clone)]
pub struct OwnedText {
    pub str_byte_range: std::ops::Range<usize>,
    pub line_infos_range: std::ops::Range<usize>,
    pub window_dim: Dimensions,
    pub font: Font,
    pub font_size: FontSize,
    pub rect: Rect,
    pub justify: Justify,
    pub y_align: Align,
    pub line_spacing: Scalar,
    pub base_line_offset: f32
}

