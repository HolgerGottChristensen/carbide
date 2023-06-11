pub use dimension::Dimension;
pub use position::Position;
pub use rect::BoundingBox;
pub use rect::Rect;

pub mod alignment;
mod dimension;
pub mod draw_gradient;
pub mod image;
pub mod path_builder;
mod position;
mod rect;
pub mod shape;
pub mod svg_path_builder;
pub mod theme;
pub mod draw_style;

/// An alias over the Scalar type used throughout carbide.
///
/// This type is primarily used for spatial dimensions and positioning.
pub type Scalar = f64;
