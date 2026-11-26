//! Screen-space outlines: see `shaders/pbr/outline.wgsl` for the method. Created on the first frame
//! with something outlined, as most scenes never are.

use encase::ShaderType;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState, ColorTargetState, CommandEncoder, Device, FragmentState, IndexFormat, Operations, PipelineLayoutDescriptor, Queue, RenderPipeline, ShaderModule, ShaderStages, StoreOp, Texture, TextureFormat, TextureSampleType, TextureView, TextureViewDimension, VertexState};

use crate::mesh_cache::MeshCache;
use crate::preprocess_shader;
use crate::render_pass_command::RenderPassCommand;
use crate::storage_buffer::StorageBuffer;
use crate::vertex::WgpuVertex;

/// Per pixel: the outline's linear colour, and its thickness in physical pixels.
const MASK_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
/// Per pixel: the offset along its row to the nearest covered pixel.
const HORIZONTAL_FORMAT: TextureFormat = TextureFormat::R32Float;
/// Marks "no covered pixel within the radius"; must match `NONE_OFFSET` in `outline.wgsl`.
const NONE_OFFSET: f64 = 10_000.0;
/// The widest search, in physical pixels: it bounds the cost of very thick outlines.
const MAX_RADIUS: i32 = 64;

#[derive(Debug, Copy, Clone, ShaderType)]
struct OutlineUniforms {
    radius: i32,
}

#[derive(Debug)]
pub(crate) struct OutlineState {
    shader: ShaderModule,
    mask_pipeline: RenderPipeline,
    horizontal_pipeline: RenderPipeline,
    horizontal_layout: BindGroupLayout,
    composite_layout: BindGroupLayout,
    /// The composite pipeline, and the layer format it was created for.
    composite_pipeline: Option<(TextureFormat, RenderPipeline)>,
    /// The mask and horizontal textures, with their size.
    targets: Option<((u32, u32), Texture, TextureView, Texture, TextureView)>,
}

fn texture_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn uniforms_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn fullscreen_pipeline(device: &Device, shader: &ShaderModule, layout: &BindGroupLayout, entry_point: &str, format: TextureFormat, blend: Option<BlendState>) -> RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("carbide_3d_outline_pipeline_layout"),
        bind_group_layouts: &[layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(entry_point),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: Some("outline_fullscreen_vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(FragmentState {
            module: shader,
            entry_point: Some(entry_point),
            compilation_options: Default::default(),
            targets: &[Some(ColorTargetState { format, blend, write_mask: wgpu::ColorWrites::ALL })],
        }),
        multiview: None,
        cache: None,
    })
}

impl OutlineState {
    /// `scene_layout` is the main pass's scene bind group layout, whose objects and camera the
    /// mask pass reads.
    pub(crate) fn new(device: &Device, scene_layout: &BindGroupLayout) -> OutlineState {
        let shader = device.create_shader_module(preprocess_shader("pbr/outline.wgsl"));

        let mask_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("carbide_3d_outline_mask_layout"),
            bind_group_layouts: &[scene_layout],
            push_constant_ranges: &[],
        });

        let mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("carbide_3d_outline_mask"),
            layout: Some(&mask_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("outline_mask_vs"),
                compilation_options: Default::default(),
                buffers: &[WgpuVertex::desc()],
            },
            // No culling and no depth: the whole silhouette, whatever is in front of it.
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("outline_mask_fs"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState { format: MASK_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });

        let horizontal_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("carbide_3d_outline_horizontal_layout"),
            entries: &[texture_entry(0), uniforms_entry(1)],
        });
        let composite_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("carbide_3d_outline_composite_layout"),
            entries: &[texture_entry(0), uniforms_entry(1), texture_entry(2)],
        });

        let horizontal_pipeline = fullscreen_pipeline(device, &shader, &horizontal_layout, "outline_horizontal", HORIZONTAL_FORMAT, None);

        OutlineState {
            shader,
            mask_pipeline,
            horizontal_pipeline,
            horizontal_layout,
            composite_layout,
            composite_pipeline: None,
            targets: None,
        }
    }

    fn ensure_targets(&mut self, device: &Device, width: u32, height: u32) {
        if self.targets.as_ref().is_some_and(|(size, ..)| *size == (width, height)) {
            return;
        }

        let create = |label: &str, format: TextureFormat| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&Default::default());
            (texture, view)
        };

        let (mask, mask_view) = create("carbide_3d_outline_mask", MASK_FORMAT);
        let (horizontal, horizontal_view) = create("carbide_3d_outline_horizontal", HORIZONTAL_FORMAT);
        self.targets = Some(((width, height), mask, mask_view, horizontal, horizontal_view));
    }

    fn ensure_composite(&mut self, device: &Device, format: TextureFormat) {
        if self.composite_pipeline.as_ref().is_some_and(|(current, _)| *current == format) {
            return;
        }

        let pipeline = fullscreen_pipeline(device, &self.shader, &self.composite_layout, "outline_composite", format, Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING));
        self.composite_pipeline = Some((format, pipeline));
    }

    /// Draw the outlines of the outlined objects among `commands` onto `target`, which already
    /// holds the scene. `objects` are the uploaded objects, whose `outline.w` is the thickness in
    /// physical pixels; `max_thickness` is the thickest of them.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        format: TextureFormat,
        (width, height): (u32, u32),
        scene_bind_group: &wgpu::BindGroup,
        commands: &[RenderPassCommand],
        outlined: &[bool],
        meshes: &MeshCache,
        max_thickness: f32,
    ) {
        self.ensure_targets(device, width, height);
        self.ensure_composite(device, format);
        let (_, _, mask_view, _, horizontal_view) = self.targets.as_ref().unwrap();

        let radius = ((max_thickness.ceil() as i32) + 1).clamp(1, MAX_RADIUS);
        let mut uniforms = StorageBuffer::from(OutlineUniforms { radius });
        uniforms.write_buffer(device, queue);

        // 1. The mask.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("carbide_3d_outline_mask"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: mask_view,
                    resolve_target: None,
                    ops: Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.mask_pipeline);
            pass.set_bind_group(0, scene_bind_group, &[]);

            for command in commands {
                if let RenderPassCommand::DrawMesh { mesh, object, .. } = command {
                    if !outlined[*object as usize] {
                        continue;
                    }
                    let Some(cached) = meshes.get(mesh.id()) else {
                        continue;
                    };

                    pass.set_vertex_buffer(0, cached.vertex_buffer.slice(..));
                    pass.set_index_buffer(cached.index_buffer.slice(..), IndexFormat::Uint32);
                    pass.draw_indexed(0..cached.index_count, 0, *object..*object + 1);
                }
            }
        }

        // 2. The row offsets.
        let horizontal_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("carbide_3d_outline_horizontal"),
            layout: &self.horizontal_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::TextureView(mask_view) },
                BindGroupEntry { binding: 1, resource: uniforms.binding().unwrap() },
            ],
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("carbide_3d_outline_horizontal"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: horizontal_view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: NONE_OFFSET, g: 0.0, b: 0.0, a: 0.0 }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.horizontal_pipeline);
            pass.set_bind_group(0, &horizontal_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        // 3. The outlines, blended onto the scene.
        let composite_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("carbide_3d_outline_composite"),
            layout: &self.composite_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::TextureView(mask_view) },
                BindGroupEntry { binding: 1, resource: uniforms.binding().unwrap() },
                BindGroupEntry { binding: 2, resource: BindingResource::TextureView(horizontal_view) },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("carbide_3d_outline_composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: Operations { load: wgpu::LoadOp::Load, store: StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.composite_pipeline.as_ref().unwrap().1);
        pass.set_bind_group(0, &composite_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
