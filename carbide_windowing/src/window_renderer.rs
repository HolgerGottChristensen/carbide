use std::error::Error;
use std::sync::Arc;

use carbide_core::draw::{Dimension, ImageContext, Scalar};
use carbide_core::environment::Environment;
use carbide_core::widget::AnyWidget;

use crate::{PhysicalSize, SurfaceTarget, WindowRenderError, WindowRendererBackend, WindowRendererFactory, WindowRendererId};

/// A window's renderer. Holds the [`WindowRendererBackend`] created by a
/// [`WindowRendererFactory`], and before every use checks that the factory in the environment is
/// still the one that created it. If not, the backend is recreated with the current factory.
pub struct WindowRenderer {
    /// The id of the factory that created the backend.
    renderer_id: WindowRendererId,
    /// What a new backend draws into, and at which size, when the backend is recreated.
    target: Arc<dyn SurfaceTarget>,
    size: PhysicalSize,
    scale_factor: Scalar,
    /// Only `None` while the backend is being replaced.
    backend: Option<Box<dyn WindowRendererBackend>>,
}

impl WindowRenderer {
    /// Create a renderer drawing into `target`, which is currently `size` physical pixels large.
    pub fn new(factory: WindowRendererFactory, target: Arc<dyn SurfaceTarget>, size: PhysicalSize, scale_factor: Scalar, env: &mut Environment) -> WindowRenderer {
        WindowRenderer {
            renderer_id: factory.id,
            backend: Some((factory.create)(target.clone(), size, scale_factor, env)),
            target,
            size,
            scale_factor,
        }
    }

    /// The id of the factory that created the current backend.
    pub fn renderer_id(&self) -> WindowRendererId {
        self.renderer_id
    }

    /// Resize the surface and every resource that depends on the window size.
    pub fn resize(&mut self, size: PhysicalSize, scale_factor: Scalar, env: &mut Environment) {
        self.size = size;
        self.scale_factor = scale_factor;

        // A recreated backend is already created at the new size.
        if !self.recreate_if_factory_changed(env) {
            self.backend().resize(size, scale_factor, env);
        }
    }

    /// Render `root`, which has already been laid out within `dimension` (logical pixels), and
    /// present the frame to the window.
    pub fn render(&mut self, root: &mut dyn AnyWidget, dimension: Dimension, scale_factor: Scalar, image: &mut dyn ImageContext, env: &mut Environment) -> Result<(), WindowRenderError> {
        self.scale_factor = scale_factor;
        self.recreate_if_factory_changed(env);

        self.backend().render(root, dimension, scale_factor, image, env)
    }

    /// Read back the last rendered frame. A backend recreated by this call has not rendered yet.
    pub fn capture(&mut self, env: &mut Environment) -> Result<image::RgbaImage, Box<dyn Error>> {
        self.recreate_if_factory_changed(env);

        self.backend().capture(env)
    }

    /// Recreate the backend if the factory in `env` is not the one that created it. Keeps the
    /// current backend if `env` has no factory. Returns whether the backend was recreated.
    fn recreate_if_factory_changed(&mut self, env: &mut Environment) -> bool {
        let Some(&factory) = env.get::<WindowRendererFactory>() else {
            return false;
        };

        if factory.id == self.renderer_id {
            return false;
        }

        // Drop the old backend before creating the new one: a window can only have one swapchain
        // at a time, so creating the new surface first fails on some platforms (e.g. Vulkan).
        self.backend = None;
        self.backend = Some((factory.create)(self.target.clone(), self.size, self.scale_factor, env));
        self.renderer_id = factory.id;

        true
    }

    fn backend(&mut self) -> &mut dyn WindowRendererBackend {
        self.backend.as_deref_mut()
            .expect("the backend is only missing while it is being replaced")
    }
}
