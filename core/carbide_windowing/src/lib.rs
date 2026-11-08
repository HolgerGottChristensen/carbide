//! The abstractions shared between windowing backends (such as carbide_winit) and renderers
//! (such as carbide_wgpu). A renderer registers a [`WindowRendererFactory`] in the environment
//! through its plugin, and the windowing backend gives each window it opens a [`WindowRenderer`],
//! so neither has to depend on the other. The `WindowRenderer` creates its
//! [`WindowRendererBackend`] with that factory, and recreates it whenever the factory in the
//! environment changes.

mod window_handle;
mod window_renderer;
mod window_renderer_backend;

pub use window_handle::*;
pub use window_renderer::*;
pub use window_renderer_backend::*;
