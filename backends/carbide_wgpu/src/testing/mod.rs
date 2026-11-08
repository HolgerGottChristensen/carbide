//! Headless rendering hooks used by `carbide_testing`. Renders into an offscreen texture instead
//! of a window surface, and exposes the draw-call plan for structural snapshots.

mod render_instructions_json;

use wgpu::CommandEncoderDescriptor;

use carbide_core::draw::{Dimension, ImageContext, Scalar};
use carbide_core::environment::Environment;
use carbide_core::render::{NoopRenderContext, RenderContext};
use carbide_core::widget::AnyWidget;
use carbide_wgpu_core::WgpuContextNew;

pub use crate::image_context::WGPUImageContext;
pub use crate::wgpu_msaa::WgpuMsaa;
use crate::offscreen_renderer::{OffscreenRenderer, TargetSize};

/// Renders a widget tree into an offscreen texture that can be read back.
#[derive(Debug)]
pub struct HeadlessRenderer {
    offscreen: OffscreenRenderer,
}

impl HeadlessRenderer {
    /// `width` and `height` are in physical pixels.
    pub fn new(width: u32, height: u32, scale_factor: Scalar, msaa: WgpuMsaa, env: &mut Environment) -> HeadlessRenderer {
        HeadlessRenderer {
            offscreen: OffscreenRenderer::new(TargetSize::new(width, height), scale_factor, msaa, env),
        }
    }

    /// `width` and `height` are in physical pixels.
    pub fn resize(&mut self, width: u32, height: u32, scale_factor: Scalar, env: &mut Environment) {
        self.offscreen.resize(TargetSize::new(width, height), scale_factor, env);
    }

    /// Render `root`, which has already been laid out within `dimension` (logical pixels), and
    /// submit the work to the GPU queue.
    pub fn render(&mut self, root: &mut dyn AnyWidget, dimension: Dimension, scale_factor: Scalar, image: &mut dyn ImageContext, env: &mut Environment) {
        let mut encoder = {
            let wgpu_context_new = env.get::<WgpuContextNew>().unwrap();
            wgpu_context_new.device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("carbide_harness_encoder"),
            })
        };

        self.offscreen.render_child(root, dimension, scale_factor, &mut RenderContext {
            render: &mut NoopRenderContext,
            image,
            env,
        }, &mut encoder);

        let wgpu_context_new = env.get::<WgpuContextNew>().unwrap();
        wgpu_context_new.queue.submit(Some(encoder.finish()));
    }

    /// Read the last rendered frame back from the GPU, via the shared
    /// [`carbide_wgpu_core::capture_texture_to_image`] readback (also used by a live window's
    /// `SceneManager::take_screenshot()` path).
    pub fn pixels(&self, env: &mut Environment) -> image::RgbaImage {
        let wgpu_context_new = env.get::<WgpuContextNew>().unwrap();
        carbide_wgpu_core::capture_texture_to_image(
            &wgpu_context_new.device,
            &wgpu_context_new.queue,
            &self.offscreen.targets[0].texture,
        ).expect("the offscreen target uses RENDER_TARGET_FORMAT, which capture_texture_to_image supports")
    }

    /// Layout-and-record only, no GPU work at all: the CPU-computed draw-call plan that `render`
    /// would otherwise hand off for GPU execution, as JSON.
    pub fn render_instructions_json(&mut self, root: &mut dyn AnyWidget, dimension: Dimension, scale_factor: Scalar, image: &mut dyn ImageContext, env: &mut Environment) -> serde_json::Value {
        let passes = self.offscreen.build_render_passes(root, dimension, scale_factor, &mut RenderContext {
            render: &mut NoopRenderContext,
            image,
            env,
        });

        render_instructions_json::render_passes_to_json(&passes)
    }
}
