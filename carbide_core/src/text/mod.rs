pub use font::*;
pub use text::Text;

pub mod font;
mod text;
mod paragraph;
mod section;

pub type FontId = usize;
pub type FontSize = u32;