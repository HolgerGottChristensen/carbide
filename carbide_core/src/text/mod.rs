//pub use markup::PolarBearMarkup;
//pub use text_span_generator::NoStyleTextSpanGenerator;
//pub use text_span_generator::TextSpanGenerator;
pub use types::font_style::FontStyle;
pub use types::font_weight::FontWeight;
pub use types::text_decoration::TextDecoration;
pub use text_context::*;
pub use text_style::*;

mod markup;
mod types;
mod text_context;
//mod text_span_generator;
mod text_style;

pub type FontId = usize;
pub type FontSize = u32;