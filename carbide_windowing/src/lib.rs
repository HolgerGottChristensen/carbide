//! The abstractions shared between windowing backends (such as carbide_winit) and renderers
//! (such as carbide_wgpu). A renderer registers a [`WindowRendererFactory`] in the environment
//! through its plugin, and the windowing backend uses it to create a [`WindowRenderer`] for each
//! window it opens, so neither has to depend on the other.

mod window_handle;
mod window_renderer;

pub use window_handle::*;
pub use window_renderer::*;
