pub use dimension::Dimension;
pub use position::Position;
pub use rect::Rect;
pub use color::Color;
pub use alignment::Alignment;
pub use image::image_context::*;
pub use texture::*;
pub use image::*;
pub use draw_gradient::*;
pub use draw_style::*;
pub use angle::*;
pub use stroke_dashes::*;

mod alignment;
mod dimension;
mod draw_gradient;
mod image;
pub mod path_builder;
mod position;
mod rect;
pub mod shape;
pub mod svg_path_builder;
pub mod theme;
mod draw_style;
pub mod color;
mod texture;
mod angle;
mod stroke_dashes;

/// An alias over the Scalar type used throughout carbide.
///
/// This type is primarily used for spatial dimensions and positioning.
pub type Scalar = f64;

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;
/// Draw colored icons from main images and not the glyph atlas.
pub const MODE_ICON: u32 = 3;
/// Draw colored bitmap glyphs.
pub const MODE_TEXT_COLOR: u32 = 4;

pub const MODE_GRADIENT_GEOMETRY: u32 = 5;

pub const MODE_GRADIENT_ICON: u32 = 6;

pub const MODE_GRADIENT_TEXT: u32 = 7;

pub const MODE_GEOMETRY_DASH_FAST: u32 = 8;
pub const MODE_GRADIENT_GEOMETRY_DASH_FAST: u32 = 9;

pub const MODE_GEOMETRY_DASH: u32 = 10;
pub const MODE_GRADIENT_GEOMETRY_DASH: u32 = 11;


/// Default dimensions to use for the glyph cache.
pub const DEFAULT_GLYPH_CACHE_DIMS: [u32; 2] = [1024; 2];
