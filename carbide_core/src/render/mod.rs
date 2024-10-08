use cgmath::Matrix4;
pub use render::*;
pub use render_context::*;
pub use noop_render_context::*;
pub use style::*;
pub use layer::*;

mod render;
mod render_context;
mod style;
mod layer;
mod noop_render_context;
pub mod triangle_render_context;

pub type CarbideTransform = Matrix4<f32>;

pub mod matrix {
    pub use cgmath::*;
}
