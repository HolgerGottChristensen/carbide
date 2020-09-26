use :: {FontSize};
use text::{Font, Justify};
use ::{Rect, Scalar};
use position::{Align, Dimensions};

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
}

