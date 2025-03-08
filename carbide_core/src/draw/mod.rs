pub use crate::misc::automatic_style::*;
pub use alignment::Alignment;
pub use angle::*;
pub use color::Color;
pub use dimension::Dimension;
pub use draw_gradient::*;
pub use draw_options::*;
pub use draw_shape::*;
pub use draw_style::*;
pub use image::image_context::*;
pub use image::*;
pub use position::Position;
pub use rect::Rect;
pub use texture::*;

mod alignment;
mod angle;
mod dimension;
mod draw_gradient;
mod draw_options;
mod draw_shape;
mod draw_style;
mod image;
mod position;
mod rect;
mod texture;
pub mod color;
pub mod fill;
pub mod path;
pub mod stroke;
pub mod theme;
pub mod gradient;

/// An alias over the Scalar type used throughout carbide.
///
/// This type is primarily used for spatial dimensions and positioning.
pub type Scalar = f64;

