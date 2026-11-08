use std::any::TypeId;
use std::error::Error;
use std::pin::Pin;
use std::sync::Arc;

use log::info;
use wgpu::{RenderPipeline, Surface, SurfaceConfiguration};

use carbide_core::draw::{Dimension, ImageContext, Scalar};
use carbide_core::environment::Environment;
use carbide_core::plugin::Plugin;
use carbide_core::register;
use carbide_core::render::{NoopRenderContext, RenderContext};
use carbide_windowing::{PhysicalSize, SurfaceTarget, WindowRenderError, WindowRenderer, WindowRendererFactory};
use carbide_core::text::TextContextKey;
use carbide_core::widget::AnyWidget;
use carbide_wgpu_core::WgpuContextNew;

use crate::image_context::WGPUImageContext;
use crate::offscreen_renderer::{OffscreenRenderer, TargetSize};
use crate::pipeline::create_final_render_pipeline;
use crate::wgpu_context::WgpuContext;
use crate::wgpu_msaa::WgpuMsaa;

/// Renders a widget tree offscreen and composites the result onto a window's surface.
pub(crate) struct WgpuWindowRenderer {
    surface: Surface<'static>,
    surface_configuration: SurfaceConfiguration,
    surface_render_pipeline: RenderPipeline,
    offscreen: OffscreenRenderer,
}

impl WgpuWindowRenderer {
    fn new(target: Arc<dyn SurfaceTarget>, size: PhysicalSize, scale_factor: Scalar, env: &mut Environment) -> WgpuWindowRenderer {
        let (wgpu_context, wgpu_context_new) = env.get_many::<(WgpuContext, WgpuContextNew)>().unwrap();

        let surface = wgpu_context_new.instance
            .create_surface(target)
            .unwrap();

        let surface_configuration = surface.get_default_config(
            &wgpu_context_new.adapter,
            size.width,
            size.height,
        ).unwrap();

        // Configure the surface with format, size and usage
        surface.configure(
            &wgpu_context_new.device,
            &surface_configuration,
        );

        info!("{:#?}", surface_configuration);

        let surface_render_pipeline = create_final_render_pipeline(
            &wgpu_context_new.device,
            &wgpu_context.texture_bind_group_layout,
            &wgpu_context.final_render_shader_srgb,
            &wgpu_context.final_render_shader_linear,
            surface_configuration.format,
        );

        let offscreen = OffscreenRenderer::new(TargetSize::new(size.width, size.height), scale_factor, WgpuMsaa::X4, env);

        WgpuWindowRenderer {
            surface,
            surface_configuration,
            surface_render_pipeline,
            offscreen,
        }
    }

    fn render_final_texture_to_swapchain(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("carbide_final_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view, // Here is the render target
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.surface_render_pipeline);
        render_pass.set_bind_group(0, &self.offscreen.targets[0].bind_group, &[]);

        render_pass.draw(0..6, 0..1);
    }
}

impl WindowRenderer for WgpuWindowRenderer {
    fn resize(&mut self, size: PhysicalSize, scale_factor: Scalar, env: &mut Environment) {
        self.offscreen.resize(TargetSize::new(size.width, size.height), scale_factor, env);

        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;

        info!("{:#?}", self.surface_configuration);

        let wgpu_context_new = env.get::<WgpuContextNew>().unwrap();

        self.surface.configure(
            &wgpu_context_new.device,
            &self.surface_configuration,
        );
    }

    fn render(&mut self, root: &mut dyn AnyWidget, dimension: Dimension, scale_factor: Scalar, image: &mut dyn ImageContext, env: &mut Environment) -> Result<(), WindowRenderError> {
        // Render the widgets into the offscreen draw-call plan
        let render_passes = self.offscreen.build_render_passes(root, dimension, scale_factor, &mut RenderContext {
            render: &mut NoopRenderContext,
            image,
            env,
        });

        let (wgpu_context, wgpu_context_new, text_context) = env.get_many::<(WgpuContext, WgpuContextNew, TextContextKey)>().unwrap();

        let mut encoder = wgpu_context_new.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("carbide_command_encoder"),
            });

        // This blocks until a new frame is available.
        let frame = self.surface.get_current_texture().map_err(|error| match error {
            wgpu::SurfaceError::Lost => WindowRenderError::Lost,
            wgpu::SurfaceError::OutOfMemory => WindowRenderError::OutOfMemory,
            error => {
                eprintln!("{:?}", error);
                WindowRenderError::Retry
            }
        })?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.offscreen.execute_render_passes(render_passes, scale_factor, &mut **text_context, wgpu_context, wgpu_context_new, &mut encoder);

        // Render from the texture to the swap chain
        self.render_final_texture_to_swapchain(
            &mut encoder,
            &view
        );

        // submit will accept anything that implements IntoIter
        wgpu_context_new.queue.submit(Some(encoder.finish()));

        frame.present();
        Ok(())
    }

    fn capture(&mut self, env: &mut Environment) -> Result<image::RgbaImage, Box<dyn Error>> {
        let wgpu_context = env.get::<WgpuContextNew>()
            .expect("WgpuContextNew is registered by its plugin before any window renders");

        Ok(carbide_wgpu_core::capture_texture_to_image(
            &wgpu_context.device,
            &wgpu_context.queue,
            &self.offscreen.targets[0].texture,
        )?)
    }
}

fn create(target: Arc<dyn SurfaceTarget>, size: PhysicalSize, scale_factor: Scalar, env: &mut Environment) -> Box<dyn WindowRenderer> {
    Box::new(WgpuWindowRenderer::new(target, size, scale_factor, env))
}

fn image_context() -> Box<dyn ImageContext> {
    Box::new(WGPUImageContext)
}

fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
    Box::pin(async {
        env.insert::<WindowRendererFactory>(Box::leak(Box::new(WindowRendererFactory {
            create,
            image_context,
        })))
    })
}

register!(Plugin {
    dependencies: &[TypeId::of::<WgpuContext>()],
    registers: TypeId::of::<WindowRendererFactory>(),
    register
});
