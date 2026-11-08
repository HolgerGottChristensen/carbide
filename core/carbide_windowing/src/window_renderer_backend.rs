use std::any::TypeId;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use raw_window_handle_06::{HasDisplayHandle, HasWindowHandle};

use carbide_core::draw::{Dimension, ImageContext, Scalar};
use carbide_core::environment::{Environment, EnvironmentKey, Shared};
use carbide_core::widget::AnyWidget;

/// A native window a renderer can draw into. Implemented for every type that exposes raw window
/// and display handles, such as `winit::window::Window`.
pub trait SurfaceTarget: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static {}

impl<T: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static> SurfaceTarget for T {}

/// A size in physical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

impl PhysicalSize {
    pub fn new(width: u32, height: u32) -> PhysicalSize {
        PhysicalSize { width, height }
    }
}

/// Why a frame could not be presented to a window.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindowRenderError {
    /// The surface was lost. The window should resize its renderer and request a redraw.
    Lost,
    /// The renderer ran out of memory. The application should close.
    OutOfMemory,
    /// The frame could not be presented this time (for example timeout or outdated).
    /// The window should request a redraw and try again next frame.
    Retry,
}

impl Display for WindowRenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRenderError::Lost => write!(f, "the window surface was lost"),
            WindowRenderError::OutOfMemory => write!(f, "the renderer ran out of memory"),
            WindowRenderError::Retry => write!(f, "the frame could not be presented, retry next frame"),
        }
    }
}

impl Error for WindowRenderError {}

/// Draws a widget tree into one native window. Implemented by renderers such as carbide_wgpu,
/// created by their [`WindowRendererFactory`], and owned by the window's
/// [`WindowRenderer`](crate::WindowRenderer).
pub trait WindowRendererBackend {
    /// Resize the surface and every resource that depends on the window size.
    fn resize(&mut self, size: PhysicalSize, scale_factor: Scalar, env: &mut Environment);

    /// Render `root`, which has already been laid out within `dimension` (logical pixels), and
    /// present the frame to the window.
    fn render(&mut self, root: &mut dyn AnyWidget, dimension: Dimension, scale_factor: Scalar, image: &mut dyn ImageContext, env: &mut Environment) -> Result<(), WindowRenderError>;

    /// Read back the last rendered frame.
    fn capture(&mut self, env: &mut Environment) -> Result<image::RgbaImage, Box<dyn Error>>;
}

/// Creates a renderer drawing into `target`, which is currently `size` physical pixels large.
pub type CreateWindowRenderer = fn(target: Arc<dyn SurfaceTarget>, size: PhysicalSize, scale_factor: Scalar, env: &mut Environment) -> Box<dyn WindowRendererBackend>;

/// Identifies a [`WindowRendererFactory`]. Two factories with the same id must create equivalent
/// renderers.
///
/// Built from a type with [`WindowRendererId::of`], usually the renderer the factory creates. The
/// factory's functions can't be compared instead: the same function can have different
/// addresses, and different functions the same address.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowRendererId(TypeId);

impl WindowRendererId {
    /// The id for factories identified by `T`.
    pub fn of<T: ?Sized + 'static>() -> WindowRendererId {
        WindowRendererId(TypeId::of::<T>())
    }
}

/// Registered into the environment by a renderer plugin, so a windowing backend can create
/// renderers for its windows without depending on the renderer.
///
/// A [`WindowRenderer`](crate::WindowRenderer) recreates its backend when the `id` of the factory
/// in the environment differs from the one that created it.
#[derive(Copy, Clone)]
pub struct WindowRendererFactory {
    /// Identifies the factory. See [`WindowRendererId`].
    pub id: WindowRendererId,
    /// Create a renderer for a window. See [`CreateWindowRenderer`].
    pub create: CreateWindowRenderer,
    /// Create the image context that widgets use to load images for this renderer.
    pub image_context: fn() -> Box<dyn ImageContext>,
}

impl Debug for WindowRendererFactory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowRendererFactory")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl EnvironmentKey for WindowRendererFactory {
    type Value = WindowRendererFactory;
    type Access = Shared;
}
