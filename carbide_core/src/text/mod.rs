pub use types::font_style::FontStyle;
pub use types::font_weight::FontWeight;
pub use types::text_decoration::TextDecoration;
pub use text_context::*;
pub use text_style::*;

mod types;
mod text_context;
mod text_style;

pub type FontId = usize;
pub type FontSize = u32;