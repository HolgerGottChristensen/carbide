//pub use text_old::Text;
use rusttype::PositionedGlyph;

pub use font::*;
pub use glyph::Glyph;
pub use text::Text;

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

pub type FontId = usize;
pub type FontSize = u32;

pub type InnerGlyph = PositionedGlyph<'static>;