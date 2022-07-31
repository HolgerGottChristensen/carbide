//pub use text_old::Text;
use rusttype::PositionedGlyph;

pub use font::*;
pub use font_family::FontFamily;
pub use types::font_style::FontStyle;
pub use types::font_weight::FontWeight;
pub use glyph::Glyph;
pub use markup::PolarBearMarkup;
pub use text::Text;
pub use types::text_decoration::TextDecoration;
pub use text_span_generator::NoStyleTextSpanGenerator;
pub use text_span_generator::TextSpanGenerator;
pub use text_style::TextStyle;

pub mod font;
mod font_family;
mod glyph;
mod markup;
mod text;
mod text_span;
mod text_span_generator;
mod text_style;
mod types;

pub type FontId = usize;
pub type FontSize = u32;

pub type InnerGlyph = PositionedGlyph<'static>;
