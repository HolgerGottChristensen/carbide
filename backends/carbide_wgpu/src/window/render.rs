use std::collections::HashMap;
use cgmath::Matrix4;
use typed_arena::Arena;
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferUsages, CommandEncoder, Device, Extent3d, ImageCopyTexture, Queue, RenderPassDepthStencilAttachment, RenderPipeline, SurfaceConfiguration, SurfaceTexture, TextureFormat, TextureUsages};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use carbide_core::application::ApplicationManager;
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Alignment, Dimension, ImageId, Position, Rect, Scalar};
use carbide_core::layout::LayoutContext;
use carbide_core::lifecycle::{Initialize, UpdateContext};
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::ReadState;
use carbide_core::text::InnerTextContext;
use carbide_core::widget::{FilterId, Widget};
use carbide_winit::convert_mouse_cursor;
use carbide_winit::dpi::PhysicalSize;
use crate::{render_pass_ops, RenderPassOps, RenderTarget, DEVICE, QUEUE};
use crate::application::ADAPTER;
use crate::bind_group_layouts::{FILTER_BUFFER_BIND_GROUP_LAYOUT, GRADIENT_DASHES_BIND_GROUP_LAYOUT, UNIFORM_BIND_GROUP_LAYOUT, UNIFORM_BIND_GROUP_LAYOUT2};
use crate::bind_groups::{filter_buffer_bind_group, gradient_dashes_bind_group, size_to_uniform_bind_group, uniforms_to_bind_group};
use crate::filter::Filter;
use crate::gradient::{Dashes, Gradient};
use crate::render_context::Uniform;
use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};
use crate::texture_atlas_command::TextureAtlasCommand;
use crate::textures::{create_depth_stencil_texture_view, create_msaa_texture_view};
use crate::vertex::Vertex;
use crate::globals::{ATLAS_CACHE_BIND_GROUP, ATLAS_CACHE_TEXTURE, BIND_GROUPS, FILTER_BIND_GROUPS, PIPELINES};
use crate::image_context::BindGroupExtended;
use crate::pipeline::RenderPipelines;
use crate::window::initialize::ZOOM;
use crate::window::initialized_window::InitializedWindow;
use crate::window::util::calculate_carbide_to_wgpu_matrix;
use crate::window::Window;

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
                self.targets.push(RenderTarget::new(self.inner.inner_size().width, self.inner.inner_size().height));
            }
        }

        //println!("\nContext: {:#?}", render_passes);
        //println!("Vertices: {:#?}", &self.render_context.vertices()[0..10]);
        //println!("Targets: {:#?}", &self.targets.len());

        let mut uniform_bind_groups = vec![];

        Self::ensure_vertices_in_buffer(&DEVICE, &QUEUE, self.render_context.vertices(), &mut self.vertex_buffer.0, &mut self.vertex_buffer.1);
        Self::ensure_uniforms_in_buffer(&DEVICE, &self.carbide_to_wgpu_matrix, self.render_context.uniforms(), &UNIFORM_BIND_GROUP_LAYOUT, &mut uniform_bind_groups);


        if self.visible {
            let cursor = convert_mouse_cursor(self.mouse_cursor);
            self.inner.set_cursor(cursor);
            self.mouse_cursor = MouseCursor::Default;

            match self.render_inner(render_passes, uniform_bind_groups, ctx.text, scale_factor) {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SurfaceError::Lost) => {
                    println!("Swap chain lost");
                    self.resize(self.inner.inner_size());
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

    fn update_atlas_cache(device: &Device, encoder: &mut CommandEncoder, ctx: &mut dyn InnerTextContext) {
        ctx.update_cache(&mut |image| {
            TextureAtlasCommand {
                texture_atlas_buffer: image.as_bytes(),
                texture_atlas_texture: &ATLAS_CACHE_TEXTURE,
                width: image.width(),
                height: image.height(),
            }.load_buffer_and_encode(device, encoder);
        });
    }

    fn update_filter_bind_groups(&self, device: &Device, filter_bind_groups: &mut HashMap<FilterId, BindGroup>, size: PhysicalSize<u32>) {
        let filters = self.render_context.filters();

        filter_bind_groups.retain(|id, _| filters.contains_key(id));

        for (filter_id, filter) in filters {
            if !filter_bind_groups.contains_key(filter_id) {
                let mut filter: Filter = filter.clone().into();
                filter.texture_size = [size.width as f32, size.height as f32];

                let filter_buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Filter Buffer"),
                    contents: &*filter.as_bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });

                let filter_buffer_bind_group = filter_buffer_bind_group(
                    device,
                    &FILTER_BUFFER_BIND_GROUP_LAYOUT,
                    &filter_buffer,
                );

                filter_bind_groups
                    .insert(*filter_id, filter_buffer_bind_group);
            }
        }
    }

    fn ensure_vertices_in_buffer(device: &Device, queue: &Queue, vertices: &Vec<Vertex>, vertex_buffer: &mut Buffer, buffer_size: &mut usize) {
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

    fn ensure_gradients_in_buffer(device: &Device, gradients: &Vec<Gradient>, _uniform_bind_group_layout: &BindGroupLayout, gradient_bind_groups: &mut Vec<BindGroup>) {
        for gradient in gradients {
            let gradient_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Gradient Buffer"),
                    contents: &*gradient.as_bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });

            let dashes = Dashes {
                dashes: [1.0; 32],
                dash_count: 2,
                start_cap: 0,
                end_cap: 0,
                total_dash_width: 2.0,
                dash_offset: 0.0,
            };
            let dashes_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Dashes Buffer"),
                    contents: &*dashes.as_bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });

            gradient_bind_groups.push(
                gradient_dashes_bind_group(
                    &device,
                    &GRADIENT_DASHES_BIND_GROUP_LAYOUT,
                    &gradient_buffer,
                    &dashes_buffer
                )
            );
        }
    }

    fn render_texture_to_swapchain(&self, encoder: &mut CommandEncoder, final_render_pipeline: &RenderPipeline, output: &SurfaceTexture, atlas_cache_bind_group: &BindGroup, bind_groups: &HashMap<ImageId, BindGroupExtended>) {
        let instance_range = 0..1;

        let (color_op, stencil_op, depth_op) = render_pass_ops(RenderPassOps::Start);

        let frame_view = output.texture.create_view(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &frame_view, // Here is the render target
                resolve_target: None,
                ops: color_op,
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(final_render_pipeline);
        render_pass.set_vertex_buffer(0, self.second_vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.targets[0].bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.gradient_dashes_bind_group, &[]);
        render_pass.set_bind_group(3, atlas_cache_bind_group, &[]);
        render_pass.set_bind_group(4, &bind_groups[&ImageId::default()].bind_group, &[]);

        render_pass.draw(0..6, instance_range);
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
        let instance_range = 0..1;
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
                                render_pass.draw(vertex_range, instance_range.clone());
                            }
                            RenderPassCommand::Stencil { vertex_range } => {
                                stencil_level += 1;
                                render_pass.set_pipeline(&render_pipelines.render_pipeline_add_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
                                current_main_render_pipeline = &render_pipelines.render_pipeline_in_mask;
                                render_pass.set_pipeline(current_main_render_pipeline);
                                render_pass.set_stencil_reference(stencil_level);
                            }
                            RenderPassCommand::DeStencil { vertex_range } => {
                                stencil_level -= 1;
                                render_pass.set_pipeline(&render_pipelines.render_pipeline_remove_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
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

                    render_pass.draw(vertex_range, instance_range.clone());
                }
                RenderPass::Clear { target_index: index } => {
                    encoder.clear_texture(&self.targets[index].texture, &Default::default())
                }
            };
        }
    }

    fn render_inner(&self, render_passes: Vec<RenderPass>, uniform_bind_groups: Vec<BindGroup>, ctx: &mut dyn InnerTextContext, scale_factor: Scalar) -> Result<(), wgpu::SurfaceError> {
        BIND_GROUPS.with(|bind_groups|  {
            let bind_groups = &*bind_groups.borrow();

            let mut encoder = DEVICE
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("carbide_command_encoder"),
                });

            let size = self.inner.inner_size();

            // Handle update of atlas cache
            Self::update_atlas_cache(&DEVICE, &mut encoder, ctx);

            // Update filter bind groups
            self.update_filter_bind_groups(&DEVICE,  &mut *FILTER_BIND_GROUPS.write().unwrap(), size);

            // Ensure the images are added as bind groups
            //Self::ensure_images_exist_as_bind_groups(device, queue, bind_groups, env);

            let pipelines = &PIPELINES.get(&self.texture_format).unwrap();


            self.process_render_passes(
                render_passes,
                &DEVICE,
                &mut encoder,
                pipelines,
                bind_groups,
                &uniform_bind_groups,
                &GRADIENT_DASHES_BIND_GROUP_LAYOUT,
                &*FILTER_BIND_GROUPS.read().unwrap(),
                size,
                scale_factor,
                &ATLAS_CACHE_BIND_GROUP
            );

            // This blocks until a new frame is available.
            let output = self.surface.get_current_texture()?;

            // Render from the texture to the swap chain
            self.render_texture_to_swapchain(&mut encoder, &pipelines.final_render_pipeline, &output, &ATLAS_CACHE_BIND_GROUP, bind_groups);

            // submit will accept anything that implements IntoIter
            QUEUE.submit(std::iter::once(encoder.finish()));

            self.inner.pre_present_notify();

            output.present();
            Ok(())
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {

        let size = new_size;
        //env.set_pixel_dimensions(size.width as f64);
        //env.set_pixel_height(size.height as f64);
        //self.ui.compound_and_add_event(Input::Redraw);

        self.depth_texture_view = create_depth_stencil_texture_view(&DEVICE, new_size.width, new_size.height, self.msaa);

        self.msaa_texture_view = create_msaa_texture_view(&DEVICE, new_size.width, new_size.height, self.msaa);

        self.targets = vec![
            RenderTarget::new(new_size.width, new_size.height)
        ];

        let scale_factor = self.inner.scale_factor();

        self.texture_size_bind_group = size_to_uniform_bind_group(
            &DEVICE,
            &UNIFORM_BIND_GROUP_LAYOUT2,
            size.width as f64,
            size.height as f64,
            scale_factor,
        );

        let dimension = Dimension::new(new_size.width as Scalar, new_size.height as Scalar);

        self.carbide_to_wgpu_matrix = calculate_carbide_to_wgpu_matrix(dimension, scale_factor);

        let uniform_bind_group = uniforms_to_bind_group(
            &DEVICE,
            &UNIFORM_BIND_GROUP_LAYOUT,
            self.carbide_to_wgpu_matrix,
            0.0,
            0.0,
            0.0,
            false
        );

        self.uniform_bind_group = uniform_bind_group;

        FILTER_BIND_GROUPS.write().unwrap().clear();

        QUEUE.write_buffer(
            &self.second_vertex_buffer,
            0,
            bytemuck::cast_slice(&Vertex::rect(size, scale_factor, ZOOM)),
        );

        let surface_caps = self.surface.get_capabilities(&*ADAPTER);

        self.surface.configure(
            &DEVICE,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: TextureFormat::Bgra8UnormSrgb,
                width: new_size.width,
                height: new_size.height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );

    }
}