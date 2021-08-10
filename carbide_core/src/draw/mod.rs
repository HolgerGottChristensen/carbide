pub use dimension::Dimension;
pub use position::Position;
pub use rect::Rect;

mod dimension;
pub mod lyon_builder_addition;
pub mod path_builder;
mod position;
mod rect;
pub mod shape;
pub mod svg_path_builder;

/// An alias over the Scalar type used throughout carbide.
///
/// This type is primarily used for spatial dimensions and positioning.
pub type Scalar = f64;
