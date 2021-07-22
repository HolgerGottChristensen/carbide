//pub use text_old::Text;
use rusttype::PositionedGlyph;

pub use font::*;
pub use font_family::FontFamily;
pub use font_style::FontStyle;
pub use font_weight::FontWeight;
pub use glyph::Glyph;
pub use markup::PolarBearMarkup;
pub use text::Text;
pub use text_decoration::TextDecoration;
pub use text_span_generator::NoStyleTextSpanGenerator;
pub use text_span_generator::TextSpanGenerator;
pub use text_style::TextStyle;

pub mod font;
mod text_old;
mod paragraph;
mod section;
mod text_span;
mod text_style;
mod text_decoration;
mod font_family;
mod font_style;
mod font_weight;
mod glyph;
mod text_overflow;
mod text;
mod markup;
mod text_span_generator;

pub type FontId = usize;
pub type FontSize = u32;

pub type InnerGlyph = PositionedGlyph<'static>;
