use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use typed_arena::Arena;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferUsages, CommandEncoder, Device, Extent3d, Queue, RenderPassDepthStencilAttachment, TexelCopyTextureInfo, Texture};
use carbide_core::draw::color_space::ColorSpace;
use carbide_core::draw::gradient::{GradientRepeat, GradientType};
use carbide_core::draw::{Dimension, DrawGradient, ImageId, Position, Rect, Scalar};
use carbide_core::environment::Environment;
use carbide_core::math::Matrix4;
use carbide_core::render::RenderContext;
use carbide_core::text::{TextContext, TextContextKey};
use carbide_core::widget::{AnyWidget, FilterId};
use carbide_wgpu_core::WgpuContextNew;

use crate::bind_groups::{filter_buffer_bind_group, gradient_dashes_bind_group, size_to_uniform_bind_group, uniforms_to_bind_group};
use crate::image_context::BindGroupExtended;
use crate::pipeline::{create_pipelines, RenderPipelines};
use crate::render_context::{Uniform, WGPURenderContext};
use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};
use crate::texture_atlas_command::TextureAtlasCommand;
use crate::wgpu_context::WgpuContext;
use crate::wgpu_dashes::WgpuDashes;
use crate::wgpu_gradient::WgpuGradient;
use crate::wgpu_msaa::WgpuMsaa;
use crate::wgpu_render_target::RENDER_TARGET_FORMAT;
use crate::wgpu_vertex::WgpuVertex;
use crate::util::calculate_carbide_to_wgpu_matrix;
use crate::{render_pass_ops, RenderPassOps, WgpuRenderTarget};

/// Everything needed to render carbide's widget tree into offscreen textures, decoupled from any
/// live OS window or `wgpu::Surface`. `InitializedWindow` holds one of these and adds only the
/// swapchain-specific bits (the surface itself, and the final blit-to-swapchain step) on top.
/// The size of the offscreen render targets, in physical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct TargetSize {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl TargetSize {
    pub(crate) fn new(width: u32, height: u32) -> TargetSize {
        TargetSize { width, height }
    }
}

pub(crate) struct OffscreenRenderer {
    size: TargetSize,
    msaa: WgpuMsaa,
    #[allow(dead_code)]
    texture_size_bind_group: BindGroup,
    pub(crate) targets: Vec<WgpuRenderTarget>,
    uniform_bind_group: BindGroup,
    gradient_buffer: Buffer,
    dashes_buffer: Buffer,
    gradient_dashes_bind_group: BindGroup,
    carbide_to_wgpu_matrix: Matrix4<f32>,
    vertex_buffer: (Buffer, usize),
    render_context: WGPURenderContext,
}

impl Debug for OffscreenRenderer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OffscreenRenderer").finish()
    }
}

impl OffscreenRenderer {
    pub(crate) fn new(size: TargetSize, scale_factor: Scalar, msaa: WgpuMsaa, env: &mut Environment) -> OffscreenRenderer {
        let pixel_dimensions = Dimension::new(size.width as Scalar, size.height as Scalar);
        let matrix = calculate_carbide_to_wgpu_matrix(pixel_dimensions, scale_factor);

        let gradient = DrawGradient {
            colors: vec![],
            ratios: vec![],
            gradient_type: GradientType::Linear,
            gradient_repeat: GradientRepeat::Clamp,
            start: Default::default(),
            end: Default::default(),
            color_space: ColorSpace::Linear,
        };
        let gradient = WgpuGradient::convert(&gradient);

        let (wgpu_context, wgpu_context_new) = env.get_many::<(WgpuContext, WgpuContextNew)>().unwrap();

        let gradient_buffer = wgpu_context_new.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("carbide_gradient_buffer"),
            contents: &gradient.as_bytes(),
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

        let dashes_buffer = wgpu_context_new.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("carbide_dashes_buffer"),
            contents: &dashes.as_bytes(),
            usage: BufferUsages::STORAGE,
        });

        let gradient_dashed_bind_group = gradient_dashes_bind_group(
            &wgpu_context_new.device,
            &wgpu_context.gradient_buffer_bind_group_layout,
            &gradient_buffer,
            &dashes_buffer,
        );

        let uniform_bind_group = uniforms_to_bind_group(
            &wgpu_context_new.device,
            &wgpu_context.uniform_bind_group_layout,
            matrix,
            0.0,
            0.0,
            0.0,
            false,
        );

        let texture_size_bind_group = size_to_uniform_bind_group(
            &wgpu_context_new.device,
            &wgpu_context.uniform_bind_group_layout2,
            pixel_dimensions.width,
            pixel_dimensions.height,
            scale_factor,
        );

        if !wgpu_context.pipelines.contains_key(&RENDER_TARGET_FORMAT) {
            let pipeline = create_pipelines(wgpu_context_new, wgpu_context, RENDER_TARGET_FORMAT, msaa);
            wgpu_context.pipelines.insert(RENDER_TARGET_FORMAT, pipeline);
        }

        let vertex_buffer = wgpu_context_new.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("carbide_primary_vertex_buffer"),
            contents: &[],
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        OffscreenRenderer {
            size,
            msaa,
            texture_size_bind_group,
            targets: vec![WgpuRenderTarget::new(size.width, size.height, msaa, env)],
            uniform_bind_group,
            gradient_buffer,
            dashes_buffer,
            gradient_dashes_bind_group: gradient_dashed_bind_group,
            carbide_to_wgpu_matrix: matrix,
            vertex_buffer: (vertex_buffer, 0),
            render_context: WGPURenderContext::new(msaa),
        }
    }

    /// Layout-and-record half of a frame: runs the child's `render()` against the internal
    /// `WGPURenderContext` and returns the resulting draw-call plan. Performs no GPU work at all
    /// (no `CommandEncoder`, no `queue.submit`) — pure CPU-side logic, safe to call standalone.
    pub(crate) fn build_render_passes(&mut self, child: &mut dyn AnyWidget, dimensions: Dimension, scale_factor: Scalar, ctx: &mut RenderContext) -> Vec<RenderPass> {
        self.render_context.start(Rect::new(Position::origin(), dimensions), scale_factor);

        TextContextKey::get_mut(ctx.env).prepare_render();

        child.render(&mut RenderContext {
            render: &mut self.render_context,
            image: ctx.image,
            env: ctx.env,
        });

        let render_passes = self.render_context.finish();

        // Most targets (the main one, mask scratch targets) are exactly window-sized, but a
        // shadow/filter scratch target may have been requested with extra `radius_x`/`radius_y`
        // padding so off-screen geometry survives to be sampled by the filter's blur passes — see
        // `WGPURenderContext::filter_new`. `target_radius` reports that requirement, already in
        // physical pixels (filters bake `scale_factor` into their own kernel taps, see
        // `Shadow::render`); (0, 0) for an ordinary target.
        let target_count = self.render_context.target_count();

        for index in 0..target_count {
            let (padding_x, padding_y) = self.render_context.target_radius(index);
            let required_width = self.size.width + 2 * padding_x;
            let required_height = self.size.height + 2 * padding_y;

            if index >= self.targets.len() {
                self.targets.push(WgpuRenderTarget::new(required_width, required_height, self.msaa, ctx.env));
            } else {
                let current_width = self.targets[index].texture.width();
                let current_height = self.targets[index].texture.height();
                if current_width < required_width || current_height < required_height {
                    self.targets[index] = WgpuRenderTarget::new(required_width, required_height, self.msaa, ctx.env);
                }
            }
        }

        render_passes
    }

    /// GPU-execution half of a frame: uploads vertex/uniform data and issues the actual draw
    /// calls for the given render-pass plan into `encoder`.
    pub(crate) fn execute_render_passes(
        &mut self,
        render_passes: Vec<RenderPass>,
        scale_factor: Scalar,
        text: &mut dyn TextContext,
        wgpu_context: &mut WgpuContext,
        wgpu_context_new: &WgpuContextNew,
        encoder: &mut CommandEncoder,
    ) {
        let mut uniform_bind_groups = vec![];

        Self::ensure_vertices_in_buffer(&wgpu_context_new.device, &wgpu_context_new.queue, self.render_context.vertices(), &mut self.vertex_buffer.0, &mut self.vertex_buffer.1);
        Self::ensure_uniforms_in_buffer(&wgpu_context_new.device, &self.carbide_to_wgpu_matrix, self.render_context.uniforms(), &wgpu_context.uniform_bind_group_layout, &mut uniform_bind_groups);

        Self::update_atlas_cache(&wgpu_context_new.device, encoder, text, &wgpu_context.atlas_cache_texture);
        self.update_filter_bind_groups(self.size, wgpu_context_new, wgpu_context);

        let pipelines = &*wgpu_context.pipelines.get(&RENDER_TARGET_FORMAT).unwrap();

        self.process_render_passes(
            render_passes,
            &wgpu_context_new.device,
            encoder,
            pipelines,
            &wgpu_context.bind_groups,
            &uniform_bind_groups,
            &wgpu_context.gradient_buffer_bind_group_layout,
            &wgpu_context.filter_bind_groups,
            self.size,
            scale_factor,
            &wgpu_context.atlas_cache_bind_group,
        );
    }

    /// Convenience combining both halves, used by the headless renderer which never needs the
    /// intermediate `Vec<RenderPass>` on its own.
    #[cfg(feature = "testing")]
    pub(crate) fn render_child(&mut self, child: &mut dyn AnyWidget, dimensions: Dimension, scale_factor: Scalar, ctx: &mut RenderContext, encoder: &mut CommandEncoder) {
        let render_passes = self.build_render_passes(child, dimensions, scale_factor, ctx);

        let (wgpu_context, wgpu_context_new, text_context) = ctx.env.get_many::<(WgpuContext, WgpuContextNew, TextContextKey)>().unwrap();
        self.execute_render_passes(render_passes, scale_factor, &mut **text_context, wgpu_context, wgpu_context_new, encoder);
    }

    pub(crate) fn resize(&mut self, new_size: TargetSize, scale_factor: Scalar, env: &mut Environment) {
        self.size = new_size;

        self.targets = vec![
            WgpuRenderTarget::new(new_size.width, new_size.height, self.msaa, env)
        ];

        let (wgpu_context, wgpu_context_new) = env.get_many::<(WgpuContext, WgpuContextNew)>().unwrap();

        self.texture_size_bind_group = size_to_uniform_bind_group(
            &wgpu_context_new.device,
            &wgpu_context.uniform_bind_group_layout2,
            new_size.width as f64,
            new_size.height as f64,
            scale_factor,
        );

        let dimension = Dimension::new(new_size.width as Scalar, new_size.height as Scalar);

        self.carbide_to_wgpu_matrix = calculate_carbide_to_wgpu_matrix(dimension, scale_factor);

        self.uniform_bind_group = uniforms_to_bind_group(
            &wgpu_context_new.device,
            &wgpu_context.uniform_bind_group_layout,
            self.carbide_to_wgpu_matrix,
            0.0,
            0.0,
            0.0,
            false,
        );

        wgpu_context.filter_bind_groups.clear();
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

    fn update_filter_bind_groups(&self, size: TargetSize, wgpu_context_new: &WgpuContextNew, wgpu_context: &mut WgpuContext) {
        let filters = self.render_context.filters();

        wgpu_context.filter_bind_groups.retain(|id, _| filters.contains_key(id));

        for (filter_id, filter) in filters {
            if !wgpu_context.filter_bind_groups.contains_key(filter_id) {
                let mut filter: crate::wgpu_filter::WgpuFilter = filter.clone().into();

                // A shadow/filter-scratch source target may be padded larger than the window (see
                // `WGPURenderContext::filter_new`) -- `texel_size` in the filter shader is derived
                // from this uniform, so it must match whichever target this specific filter
                // actually samples from, not always the window's own size.
                //
                // This is plain physical pixels, matching the filter's own kernel taps: a
                // filter's offsets/sigma already have `scale_factor` baked in at the point they're
                // built (see `Shadow::render`), so `filter`/`texture_size` here are a pure
                // function of the `ImageFilter`'s own content -- nothing scale-factor-dependent is
                // smuggled in through this uniform, keeping a filter's bind group correctly
                // reusable for as long as its `FilterId` is alive.
                let texture_size = self.render_context.filter_source_target(*filter_id)
                    .map(|index| {
                        let target = &self.targets[index];
                        (target.texture.width() as f32, target.texture.height() as f32)
                    })
                    .unwrap_or((size.width as f32, size.height as f32));

                filter.texture_size = [texture_size.0, texture_size.1];

                let filter_buffer = wgpu_context_new.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Filter Buffer"),
                    contents: &filter.as_bytes(),
                    usage: BufferUsages::STORAGE,
                });

                let filter_buffer_bind_group = filter_buffer_bind_group(
                    &wgpu_context_new.device,
                    &wgpu_context.filter_buffer_bind_group_layout,
                    &filter_buffer,
                );

                wgpu_context.filter_bind_groups
                    .insert(*filter_id, filter_buffer_bind_group);
            }
        }
    }

    fn ensure_vertices_in_buffer(device: &Device, queue: &Queue, vertices: &[WgpuVertex], vertex_buffer: &mut Buffer, buffer_size: &mut usize) {
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

    #[allow(dead_code)]
    fn ensure_gradients_in_buffer(device: &Device, gradients: &Vec<WgpuGradient>, gradient_bind_groups: &mut Vec<BindGroup>, gradient_dashes_bind_group_layout: &BindGroupLayout) {
        for gradient in gradients {
            let gradient_buffer =
                device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Gradient Buffer"),
                    contents: &gradient.as_bytes(),
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
                    contents: &dashes.as_bytes(),
                    usage: BufferUsages::STORAGE,
                });

            gradient_bind_groups.push(
                gradient_dashes_bind_group(
                    device,
                    gradient_dashes_bind_group_layout,
                    &gradient_buffer,
                    &dashes_buffer
                )
            );
        }
    }

    fn process_render_passes(
        &self,
        render_passes: Vec<RenderPass>,
        device: &Device,
        encoder: &mut CommandEncoder,
        render_pipelines: &RenderPipelines,
        bind_groups: &HashMap<ImageId, BindGroupExtended>,
        uniform_bind_groups: &[BindGroup],
        gradient_dashes_bind_group_layout: &BindGroupLayout,
        filter_bind_groups: &HashMap<FilterId, BindGroup>,
        size: TargetSize,
        scale_factor: Scalar,
        atlas_cache_bind_group: &BindGroup,
    ) {
        let mut stencil_level = 0;

        let mut target_started = vec![false; self.targets.len()];

        let mut current_target_view;
        let mut current_main_render_pipeline = &render_pipelines.render_pipeline_no_mask;
        let current_vertex_buffer_slice = self.vertex_buffer.0.slice(..);
        let mut current_uniform_bind_group = &self.uniform_bind_group;
        let mut current_gradient_dashes_bind_group = &self.gradient_dashes_bind_group;
        let mut current_gradient_buffer = &self.gradient_buffer;
        let mut current_dashes_buffer = &self.dashes_buffer;
        let mut invalid_scissor = false;

        let buffers = Arena::new();
        let gradient_dashed_bind_groups = Arena::new();

        for command in render_passes {
            match command {
                RenderPass::Normal { commands: inner, target_index: index } => {
                    current_target_view = &self.targets[index].view;

                    if inner.is_empty() {
                        continue;
                    }

                    let (color_op, stencil_op, depth_op) = if target_started[index] {
                        render_pass_ops(RenderPassOps::Middle)
                    } else {
                        target_started[index] = true;
                        render_pass_ops(RenderPassOps::Start)
                    };

                    let (view, resolve_target) = self.targets[index].msaa_view.as_ref()
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
                            view: &self.targets[index].depth_view,
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

                                invalid_scissor = width == 0 || height == 0;

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
                                        contents: &gradient.as_bytes(),
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
                                        contents: &dashes.as_bytes(),
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
                            TexelCopyTextureInfo {
                                texture: &self.targets[target_id].texture,
                                mip_level: 0,
                                origin: Default::default(),
                                aspect: Default::default(),
                            },
                            TexelCopyTextureInfo {
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

                    let (color_op, stencil_op, depth_op) = if target_started[target_id] {
                        render_pass_ops(RenderPassOps::Middle)
                    } else {
                        target_started[target_id] = true;
                        render_pass_ops(RenderPassOps::Start)
                    };

                    let (view, resolve_target) = self.targets[target_id].msaa_view
                        .as_ref()
                        .map(|a| (a, Some(&self.targets[target_id].view)))
                        .unwrap_or((&self.targets[target_id].view, None));

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view, // Here is the render target
                            resolve_target,
                            ops: color_op,
                        })],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.targets[target_id].depth_view,
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
}
