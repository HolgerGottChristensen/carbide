pub use dimension::Dimension;
pub use position::Position;
pub use rect::Rect;

pub mod shape;
pub mod lyon_builder_addition;
pub mod path_builder;
pub mod svg_path_builder;
mod rect;
mod dimension;
mod position;


/// An alias over the Scalar type used throughout carbide.
///
/// This type is primarily used for spatial dimensions and positioning.
pub type Scalar = f64;

/// General use 2D spatial dimensions.
pub type Dimensions = [Scalar; 2];

/// General use 2D spatial point.
pub type Point = [Scalar; 2];

