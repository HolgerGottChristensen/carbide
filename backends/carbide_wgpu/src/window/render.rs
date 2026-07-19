use crate::render_pass_command::RenderPass;
use crate::window::initialized_window::InitializedWindow;
use crate::window::Window;
use carbide_core::application::ApplicationManager;
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Alignment, Dimension, Position, Rect, Scalar};
use carbide_core::layout::LayoutContext;
use carbide_core::lifecycle::UpdateContext;
use carbide_core::render::{NoopRenderContext, Render, RenderContext};
use carbide_core::state::ReadState;
use carbide_core::text::TextContext;
use carbide_core::widget::Widget;
use carbide_winit::convert_mouse_cursor;
use carbide_winit::dpi::PhysicalSize;
use log::info;
use carbide_core::environment::Environment;
use crate::wgpu_context::WgpuContext;

impl<T: ReadState<T=String>, C: Widget> Render for Window<T, C> {
    fn render(&mut self, ctx: &mut RenderContext) {
        match self {
            Window::Initialized(initialized) => initialized.render(ctx),
            Window::UnInitialized { .. } => {}
            Window::Failed => {}
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> InitializedWindow<T, C> {
    fn render(&mut self, ctx: &mut RenderContext) {
        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();
        let logical_dimensions = physical_dimensions.to_logical(scale_factor);
        let dimensions = Dimension::new(logical_dimensions.width, logical_dimensions.height);

        info!("Render window, {}, {}", scale_factor, dimensions);

        let mut render_passes: Option<Vec<RenderPass>> = None;

        self.with_env(ctx.env, |env, initialized| {
            for scene in &mut initialized.scenes {
                scene.render(&mut RenderContext {
                    render: ctx.render,
                    text: ctx.text,
                    image: ctx.image,
                    env,
                });
            }

            // Update children
            initialized.child.process_update(&mut UpdateContext {
                text: ctx.text,
                image: ctx.image,
                env,
            });

            // Calculate size
            initialized.child.calculate_size(dimensions, &mut LayoutContext {
                text: ctx.text,
                image: ctx.image,
                env,
            });

            // Position children
            let alignment = Alignment::Center;
            initialized.child.set_position(alignment.position(Position::new(0.0, 0.0), dimensions, initialized.child.dimension()));

            // When we have a window, everything outside that window can be clipped when rendered.
            let bounding_box = Rect::new(
                Position::new(0.0, 0.0),
                dimensions
            );

            initialized.child.position_children(
                bounding_box,
                &mut LayoutContext {
                    text: ctx.text,
                    image: ctx.image,
                    env,
                }
            );

            // Render the children into the offscreen draw-call plan
            render_passes = Some(initialized.offscreen.build_render_passes(&mut initialized.child, dimensions, scale_factor, &mut RenderContext {
                render: &mut NoopRenderContext,
                text: ctx.text,
                image: ctx.image,
                env,
            }));

            if initialized.visible {
                {
                    initialized.title.sync(env);

                    let current = &*initialized.title.value();
                    if &initialized.inner.title() != current {
                        initialized.inner.set_title(current);
                    }
                }
            }
        });

        let render_passes = render_passes.expect("with_env always runs its closure exactly once");

        let wgpu_context = ctx.env.get_mut::<WgpuContext>().unwrap();

        if self.visible {
            let cursor = convert_mouse_cursor(self.mouse_cursor);
            self.inner.set_cursor(cursor);
            self.mouse_cursor = MouseCursor::Default;

            match self.render_inner(render_passes, ctx.text, scale_factor, wgpu_context) {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SurfaceError::Lost) => {
                    println!("Swap chain lost");
                    self.resize(self.inner.inner_size(), ctx.env);
                    self.inner.request_redraw();
                }
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    println!("Swap chain out of memory");
                    ApplicationManager::get(ctx.env, |manager| {
                        manager.close();
                    });
                }
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => {
                    // We request a redraw the next frame
                    self.inner.request_redraw();
                    eprintln!("{:?}", e)
                }
            }
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

    fn render_inner(
        &mut self,
        render_passes: Vec<RenderPass>,
        ctx: &mut dyn TextContext,
        scale_factor: Scalar,
        wgpu_context: &mut WgpuContext,
    ) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = wgpu_context.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("carbide_command_encoder"),
            });

        // This blocks until a new frame is available.
        let frame = self.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.offscreen.execute_render_passes(render_passes, scale_factor, ctx, wgpu_context, &mut encoder);

        // Render from the texture to the swap chain
        self.render_final_texture_to_swapchain(
            &mut encoder,
            &view
        );

        // submit will accept anything that implements IntoIter
        wgpu_context.queue.submit(Some(encoder.finish()));

        self.inner.pre_present_notify();

        frame.present();
        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>, env: &mut Environment) {
        let scale_factor = self.inner.scale_factor();

        self.offscreen.resize(new_size, scale_factor, env);

        self.surface_configuration.width = new_size.width;
        self.surface_configuration.height = new_size.height;

        info!("{:#?}", self.surface_configuration);

        let wgpu_context = env.get_mut::<WgpuContext>().unwrap();

        self.surface.configure(
            &wgpu_context.device,
            &self.surface_configuration,
        );
    }
}
