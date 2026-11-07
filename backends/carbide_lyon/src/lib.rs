mod tesselator;
mod plugin;

pub use tesselator::*;

// Re-exported for backwards compatibility. These types now live in carbide_tesselation.
pub use carbide_tesselation::{triangle, vertex, stroke_vertex};

pub use lyon::geom::Scalar as LyonScalar;
