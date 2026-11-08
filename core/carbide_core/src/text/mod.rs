pub use types::font_style::FontStyle;
pub use types::font_weight::FontWeight;
pub use text_decoration::TextDecoration;
pub use text_context::*;
pub use text_style::*;

mod types;
mod text_context;
mod text_style;
pub mod text_wrap;
pub mod text_justify;
pub mod text_decoration;
pub mod glyph;

pub type FontId = usize;
pub type FontSize = u32;