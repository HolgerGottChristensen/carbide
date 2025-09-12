use crate::bind_groups::{filter_buffer_bind_group, gradient_dashes_bind_group, size_to_uniform_bind_group, uniforms_to_bind_group};
use crate::wgpu_filter::WgpuFilter;
use crate::wgpu_gradient::{WgpuGradient};
use crate::image_context::BindGroupExtended;
use crate::pipeline::RenderPipelines;
use crate::render_context::Uniform;
use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};
use crate::texture_atlas_command::TextureAtlasCommand;
use crate::textures::{create_depth_stencil_texture_view, create_msaa_texture_view};
use crate::wgpu_vertex::WgpuVertex;
use crate::window::initialize::ZOOM;
use crate::window::initialized_window::InitializedWindow;
use crate::window::util::calculate_carbide_to_wgpu_matrix;
use crate::window::Window;
use crate::{render_pass_ops, RenderPassOps, WgpuRenderTarget};
use carbide_core::application::ApplicationManager;
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Alignment, Dimension, ImageId, Position, Rect, Scalar};
use carbide_core::layout::LayoutContext;
use carbide_core::lifecycle::{Initialize, UpdateContext};
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::ReadState;
use carbide_core::text::TextContext;
use carbide_core::widget::{FilterId, Widget};
use carbide_winit::convert_mouse_cursor;
use carbide_winit::dpi::PhysicalSize;
use std::collections::HashMap;
use log::info;
use typed_arena::Arena;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferUsages, CommandEncoder, Device, Extent3d, ImageCopyTexture, LoadOp, Operations, Queue, RenderPassDepthStencilAttachment, RenderPipeline, StoreOp, SurfaceConfiguration, SurfaceTexture, Texture, TextureFormat, TextureUsages, TextureView};
use carbide_core::environment::Environment;
use carbide_core::math::Matrix4;
use crate::wgpu_render_target::RENDER_TARGET_FORMAT;
use crate::wgpu_context::WgpuContext;
use crate::wgpu_dashes::WgpuDashes;

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
            initialized.child.position_children(&mut LayoutContext {
                text: ctx.text,
                image: ctx.image,
                env,
            });

            // Render the children
            initialized.render_context.start(Rect::new(Position::origin(), dimensions));

            ctx.text.prepare_render();

            initialized.child.render(&mut RenderContext {
                render: &mut initialized.render_context,
                text: ctx.text,
                image: ctx.image,
                env,
            });

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

        let render_passes = self.render_context.finish();

        let target_count = self.render_context.target_count();
        if self.targets.len() < target_count {
            for _ in self.targets.len()..target_count {
                self.targets.push(WgpuRenderTarget::new(self.inner.inner_size().width, self.inner.inner_size().height, ctx.env));
            }
        }

        //println!("\nContext: {:#?}", render_passes);
        //println!("Vertices: {:#?}", &self.render_context.vertices()[0..10]);
        //println!("Targets: {:#?}", &self.targets.len());

        let wgpu_context = ctx.env.get_mut::<WgpuContext>().unwrap();

        let mut uniform_bind_groups = vec![];

        Self::ensure_vertices_in_buffer(&wgpu_context.device, &wgpu_context.queue, self.render_context.vertices(), &mut self.vertex_buffer.0, &mut self.vertex_buffer.1);
        Self::ensure_uniforms_in_buffer(&wgpu_context.device, &self.carbide_to_wgpu_matrix, self.render_context.uniforms(), &wgpu_context.uniform_bind_group_layout, &mut uniform_bind_groups);


        if self.visible {
            let cursor = convert_mouse_cursor(self.mouse_cursor);
            self.inner.set_cursor(cursor);
            self.mouse_cursor = MouseCursor::Default;

            match self.render_inner(render_passes, uniform_bind_groups, ctx.text, scale_factor, wgpu_context) {
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

    fn update_atlas_cache(device: &Device, encoder: &mut CommandEncoder, ctx: &mut dyn TextContext, texture: &Texture) {
        ctx.update_cache(&mut |image| {
            TextureAtlasCommand {
                texture_atlas_buffer: image.as_bytes(),
                texture_atlas_texture: texture,
                width: image.width(),
                height: image.height(),
            }.load_buffer_and_encode(device, encoder);
        });
    }

    fn update_filter_bind_groups(&self, size: PhysicalSize<u32>, wgpu_context: &mut WgpuContext) {
        let filters = self.render_context.filters();

        wgpu_context.filter_bind_groups.retain(|id, _| filters.contains_key(id));

        for (filter_id, filter) in filters {
            if !wgpu_context.filter_bind_groups.contains_key(filter_id) {
                let mut filter: WgpuFilter = filter.clone().into();
                filter.texture_size = [size.width as f32, size.height as f32];

                let filter_buffer = wgpu_context.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Filter Buffer"),
                    contents: &*filter.as_bytes(),
                    usage: BufferUsages::STORAGE,
                });

                let filter_buffer_bind_group = filter_buffer_bind_group(
                    &wgpu_context.device,
                    &wgpu_context.filter_buffer_bind_group_layout,
                    &filter_buffer,
                );

                wgpu_context.filter_bind_groups
                    .insert(*filter_id, filter_buffer_bind_group);
            }
        }
    }

    fn ensure_vertices_in_buffer(device: &Device, queue: &Queue, vertices: &Vec<WgpuVertex>, vertex_buffer: &mut Buffer, buffer_size: &mut usize) {
        if vertices.len() <= *buffer_size {
            // There is space in the current vertex buffer
            queue.write_buffer(vertex_buffer, 0, bytemuck::cast_slice(vertices));
        } else {
            // We need to create a new and larger vertex buffer
            let new_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            *vertex_buffer = new_vertex_buffer;
            *buffer_size = vertices.len();
        }
    }

    fn ensure_uniforms_in_buffer(device: &Device, carbide_to_wgpu_matrix: &Matrix4<f32>, uniforms: &Vec<Uniform>, uniform_bind_group_layout: &BindGroupLayout, uniform_bind_groups: &mut Vec<BindGroup>) {
        for uniform in uniforms {
            let transformed_matrix = carbide_to_wgpu_matrix * uniform.transform;

            let new_bind_group = uniforms_to_bind_group(
                device,
                uniform_bind_group_layout,
                transformed_matrix,
                uniform.hue_rotation,
                uniform.saturation_shift,
                uniform.luminance_shift,
                uniform.color_invert,
            );

            uniform_bind_groups.push(new_bind_group);
        }
    }

    fn ensure_gradients_in_buffer(device: &Device, gradients: &Vec<WgpuGradient>, gradient_bind_groups: &mut Vec<BindGroup>, gradient_dashes_bind_group_layout: &BindGroupLayout) {
        for gradient in gradients {
            let gradient_buffer =
                device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Gradient Buffer"),
                    contents: &*gradient.as_bytes(),
                    usage: BufferUsages::STORAGE,
                });

            let dashes = WgpuDashes {
                dashes: [1.0; 32],
                dash_count: 2,
                start_cap: 0,
                end_cap: 0,
                total_dash_width: 2.0,
                dash_offset: 0.0,
            };
            let dashes_buffer =
                device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Dashes Buffer"),
                    contents: &*dashes.as_bytes(),
                    usage: BufferUsages::STORAGE,
                });

            gradient_bind_groups.push(
                gradient_dashes_bind_group(
                    &device,
                    gradient_dashes_bind_group_layout,
                    &gradient_buffer,
                    &dashes_buffer
                )
            );
        }
    }

    fn render_final_texture_to_swapchain(&self, encoder: &mut CommandEncoder, view: &TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("carbide_final_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view, // Here is the render target
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.surface_render_pipeline);
        render_pass.set_bind_group(0, &self.targets[0].bind_group, &[]);


        render_pass.draw(0..6, 0..1);
    }

    fn process_render_passes(
        &self,
        render_passes: Vec<RenderPass>,
        device: &Device,
        encoder: &mut CommandEncoder,
        render_pipelines: &RenderPipelines,
        bind_groups: &HashMap<ImageId, BindGroupExtended>,
        uniform_bind_groups: &Vec<BindGroup>,
        gradient_dashes_bind_group_layout: &BindGroupLayout,
        filter_bind_groups: &HashMap<FilterId, BindGroup>,
        size: PhysicalSize<u32>,
        scale_factor: Scalar,
        atlas_cache_bind_group: &BindGroup,
    ) {
        let mut stencil_level = 0;
        let mut first_pass = true;

        let mut current_target_view;
        let mut current_main_render_pipeline = &render_pipelines.render_pipeline_no_mask;
        let current_vertex_buffer_slice = self.vertex_buffer.0.slice(..);
        let mut current_uniform_bind_group = &self.uniform_bind_group;
        let mut current_gradient_dashes_bind_group = &self.gradient_dashes_bind_group;
        let mut current_gradient_buffer = &self.gradient_buffer;
        let mut current_dashes_buffer = &self.dashes_buffer;
        let mut invalid_scissor = false;

        let mut buffers = Arena::new();
        let mut gradient_dashed_bind_groups = Arena::new();

        // println!("{:#?}", render_passes);

        for command in render_passes {
            match command {
                RenderPass::Normal { commands: inner, target_index: index } => {
                    current_target_view = &self.targets[index].view;

                    if inner.len() == 0 {
                        continue;
                    }

                    let (color_op, stencil_op, depth_op) = if first_pass {
                        first_pass = false;
                        render_pass_ops(RenderPassOps::Start)
                    } else {
                        render_pass_ops(RenderPassOps::Middle)
                    };

                    let (view, resolve_target) = self.msaa_texture_view.as_ref()
                        .map(|a| (a, Some(current_target_view)))
                        .unwrap_or((current_target_view, None));

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view, // Here is the render target
                            resolve_target,
                            ops: color_op,
                        })],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: Some(depth_op),
                            stencil_ops: Some(stencil_op),
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_pipeline(current_main_render_pipeline);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &bind_groups[&ImageId::default()].bind_group, &[]);
                    render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(2, current_gradient_dashes_bind_group, &[]);
                    render_pass.set_bind_group(3, atlas_cache_bind_group, &[]);
                    render_pass.set_bind_group(4, &bind_groups[&ImageId::default()].bind_group, &[]);


                    for inner_command in inner {
                        match inner_command {
                            RenderPassCommand::SetBindGroup { bind_group } => {
                                match bind_group {
                                    WGPUBindGroup::Image(id) => {
                                        render_pass.set_bind_group(0, &bind_groups[&id].bind_group, &[]);
                                    }
                                    WGPUBindGroup::Target(index) => {
                                        render_pass.set_bind_group(0, &self.targets[index].bind_group, &[]);
                                    }
                                    WGPUBindGroup::Layer(id) => {
                                        render_pass.set_bind_group(0, self.render_context.layer_bind_group(id), &[])
                                    }
                                }
                            }
                            RenderPassCommand::SetScissor { rect } => {
                                let x = (rect.left() * scale_factor).max(0.0) as u32;
                                let y = (rect.bottom() * scale_factor).max(0.0) as u32;
                                let width = (rect.width() * scale_factor) as u32;
                                let height = (rect.height() * scale_factor) as u32;

                                invalid_scissor = width <= 0 || height <= 0;

                                if !invalid_scissor {
                                    render_pass.set_scissor_rect(x, y, width, height);
                                }
                            }
                            RenderPassCommand::Draw { vertex_range } => {
                                if invalid_scissor {
                                    continue;
                                }
                                render_pass.draw(vertex_range, 0..1);
                            }
                            RenderPassCommand::Stencil { vertex_range } => {
                                stencil_level += 1;
                                render_pass.set_pipeline(&render_pipelines.render_pipeline_add_mask);
                                render_pass.draw(vertex_range, 0..1);
                                current_main_render_pipeline = &render_pipelines.render_pipeline_in_mask;
                                render_pass.set_pipeline(current_main_render_pipeline);
                                render_pass.set_stencil_reference(stencil_level);
                            }
                            RenderPassCommand::DeStencil { vertex_range } => {
                                stencil_level -= 1;
                                render_pass.set_pipeline(&render_pipelines.render_pipeline_remove_mask);
                                render_pass.draw(vertex_range, 0..1);
                                render_pass.set_stencil_reference(stencil_level);
                                if stencil_level == 0 {
                                    current_main_render_pipeline = &render_pipelines.render_pipeline_no_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                } else {
                                    current_main_render_pipeline = &render_pipelines.render_pipeline_in_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                }
                            }
                            RenderPassCommand::Uniform {
                                uniform_bind_group_index,
                            } => {
                                current_uniform_bind_group = &uniform_bind_groups[uniform_bind_group_index];
                                render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                            }
                            RenderPassCommand::Gradient(gradient) => {
                                current_gradient_buffer =
                                    buffers.alloc(device.create_buffer_init(&BufferInitDescriptor {
                                        label: Some("Gradient Buffer"),
                                        contents: &*gradient.as_bytes(),
                                        usage: BufferUsages::STORAGE,
                                    }));

                                current_gradient_dashes_bind_group = gradient_dashed_bind_groups.alloc(gradient_dashes_bind_group(
                                    device,
                                    gradient_dashes_bind_group_layout,
                                    current_gradient_buffer,
                                    current_dashes_buffer,
                                ));

                                render_pass.set_bind_group(2, current_gradient_dashes_bind_group, &[]);
                            }
                            RenderPassCommand::StrokeDashing(dashes) => {
                                current_dashes_buffer =
                                    buffers.alloc(device.create_buffer_init(&BufferInitDescriptor {
                                        label: Some("Dashes Buffer"),
                                        contents: &*dashes.as_bytes(),
                                        usage: BufferUsages::STORAGE,
                                    }));

                                current_gradient_dashes_bind_group = gradient_dashed_bind_groups.alloc(gradient_dashes_bind_group(
                                    device,
                                    gradient_dashes_bind_group_layout,
                                    current_gradient_buffer,
                                    current_dashes_buffer,
                                ));

                                render_pass.set_bind_group(2, current_gradient_dashes_bind_group, &[]);
                            }
                            RenderPassCommand::SetMaskBindGroup { bind_group } => {
                                match bind_group {
                                    WGPUBindGroup::Image(id) => {
                                        render_pass.set_bind_group(4, &bind_groups[&id].bind_group, &[]);
                                    }
                                    WGPUBindGroup::Target(index) => {
                                        render_pass.set_bind_group(4, &self.targets[index].bind_group, &[]);
                                    }
                                    WGPUBindGroup::Layer(id) => {
                                        render_pass.set_bind_group(4, self.render_context.layer_bind_group(id), &[]);
                                    }
                                }
                            }
                        }
                    }
                }
                RenderPass::Filter {
                    vertex_range,
                    filter_id,
                    source_id,
                    target_id,
                    mask_id,
                    initial_copy
                } => {
                    if invalid_scissor {
                        continue;
                    }

                    if initial_copy {
                        encoder.copy_texture_to_texture(
                            ImageCopyTexture {
                                texture: &self.targets[target_id].texture,
                                mip_level: 0,
                                origin: Default::default(),
                                aspect: Default::default(),
                            },
                            ImageCopyTexture {
                                texture: &self.targets[source_id].texture,
                                mip_level: 0,
                                origin: Default::default(),
                                aspect: Default::default(),
                            },
                            Extent3d {
                                width: size.width,
                                height: size.height,
                                depth_or_array_layers: 1,
                            },
                        );
                    }

                    let (color_op, stencil_op, depth_op) = render_pass_ops(RenderPassOps::Middle);

                    let (view, resolve_target) = self.msaa_texture_view.as_ref().map(|a| (a, Some(&self.targets[target_id].view))).unwrap_or((&self.targets[target_id].view, None));

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view, // Here is the render target
                            resolve_target,
                            ops: color_op,
                        })],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: Some(depth_op),
                            stencil_ops: Some(stencil_op),
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    render_pass.set_pipeline(&render_pipelines.render_pipeline_in_mask_filter);
                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &self.targets[source_id].bind_group, &[]);
                    render_pass.set_bind_group(1, &filter_bind_groups[&filter_id], &[]);
                    render_pass.set_bind_group(2, current_uniform_bind_group, &[]);
                    if let Some(id) = mask_id {
                        render_pass.set_bind_group(3, &self.targets[id].bind_group, &[]);
                    } else {
                        render_pass.set_bind_group(3, &bind_groups[&ImageId::default()].bind_group, &[]);
                    }

                    render_pass.draw(vertex_range, 0..1);
                }
                RenderPass::Clear { target_index: index } => {
                    encoder.clear_texture(&self.targets[index].texture, &Default::default())
                }
            }
        }
    }

    fn render_inner(
        &self,
        render_passes: Vec<RenderPass>,
        uniform_bind_groups: Vec<BindGroup>,
        ctx: &mut dyn TextContext,
        scale_factor: Scalar,
        wgpu_context: &mut WgpuContext
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

        let size = self.inner.inner_size();

        info!("{:?}", size);

        // Handle update of atlas cache
        Self::update_atlas_cache(&wgpu_context.device, &mut encoder, ctx, &wgpu_context.atlas_cache_texture);

        // Update filter bind groups
        self.update_filter_bind_groups(size, wgpu_context);

        info!("{:#?}", &self.surface_configuration.format);

        info!("{:#?}", render_passes);

        if false {
            self.test(&mut encoder, wgpu_context, &view);
        } else {
            let pipelines = &*wgpu_context.pipelines.get(&RENDER_TARGET_FORMAT).unwrap();

            self.process_render_passes(
                render_passes,
                &wgpu_context.device,
                &mut encoder,
                pipelines,
                &wgpu_context.bind_groups,
                &uniform_bind_groups,
                &wgpu_context.gradient_buffer_bind_group_layout,
                &wgpu_context.filter_bind_groups,
                size,
                scale_factor,
                &wgpu_context.atlas_cache_bind_group
            );
        }

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

    pub fn test(&self, encoder: &mut CommandEncoder, wgpu_context: &mut WgpuContext, view: &TextureView) {
        let view = &self.targets[0].view;

        let render_pipeline = wgpu_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blalbalba"),
            layout: Some(&wgpu_context.main_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &wgpu_context.main_shader,
                entry_point: Some("main_vs"),
                buffers: &[WgpuVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &wgpu_context.main_shader,
                entry_point: Some("main_fs"),
                compilation_options: Default::default(),
                targets: &[Some(RENDER_TARGET_FORMAT.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let mut rpass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.2,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        let mut current_uniform_bind_group = &self.uniform_bind_group;
        let mut current_gradient_dashes_bind_group = &self.gradient_dashes_bind_group;

        rpass.set_bind_group(0, &wgpu_context.bind_groups[&ImageId::default()].bind_group, &[]);
        rpass.set_bind_group(1, current_uniform_bind_group, &[]);
        rpass.set_bind_group(2, current_gradient_dashes_bind_group, &[]);
        rpass.set_bind_group(3, &wgpu_context.atlas_cache_bind_group, &[]);
        rpass.set_bind_group(4, &wgpu_context.bind_groups[&ImageId::default()].bind_group, &[]);

        rpass.set_pipeline(&render_pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.0.slice(..));
        rpass.draw(6..36, 0..1);
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>, env: &mut Environment) {

        let size = new_size;

        self.targets = vec![
            WgpuRenderTarget::new(new_size.width, new_size.height, env)
        ];

        let wgpu_context = env.get_mut::<WgpuContext>().unwrap();

        self.depth_texture_view = create_depth_stencil_texture_view(&wgpu_context.device, new_size.width, new_size.height, self.msaa);

        self.msaa_texture_view = create_msaa_texture_view(&wgpu_context.device, new_size.width, new_size.height, self.msaa);

        let scale_factor = self.inner.scale_factor();

        self.texture_size_bind_group = size_to_uniform_bind_group(
            &wgpu_context.device,
            &wgpu_context.uniform_bind_group_layout2,
            size.width as f64,
            size.height as f64,
            scale_factor,
        );

        let dimension = Dimension::new(new_size.width as Scalar, new_size.height as Scalar);

        self.carbide_to_wgpu_matrix = calculate_carbide_to_wgpu_matrix(dimension, scale_factor);

        let uniform_bind_group = uniforms_to_bind_group(
            &wgpu_context.device,
            &wgpu_context.uniform_bind_group_layout,
            self.carbide_to_wgpu_matrix,
            0.0,
            0.0,
            0.0,
            false
        );

        self.uniform_bind_group = uniform_bind_group;

        wgpu_context.filter_bind_groups.clear();

        self.surface_configuration.width = new_size.width;
        self.surface_configuration.height = new_size.height;

        info!("{:#?}", self.surface_configuration);

        self.surface.configure(
            &wgpu_context.device,
            &self.surface_configuration,
        );

    }
}