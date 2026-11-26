//! The picking buffer: rendered with a frame when the view asks for it, kept until the next frame,
//! and read around the pointer on demand.
//!
//! A pick copies the 15 × 15 tile around the pointer into a small mappable buffer and waits for it
//! (`PollType::Wait`). The GPU has usually finished the frame by the time a pointer event
//! arrives, so the wait is short. Textures can't be mapped, so a read is always a copy into a
//! buffer, then a map.

use carbide_3d::render::PickSample;
use carbide_core::math::{InnerSpace, Matrix4, SquareMatrix, Vector3, Vector4};
use wgpu::{BindGroupLayout, Buffer, CommandEncoder, CompareFunction, DepthStencilState, Device, Face, FragmentState, FrontFace, IndexFormat, Operations, PrimitiveState, Queue, RenderPipeline, StoreOp, Texture, TextureFormat, TextureView, VertexState};

use crate::mesh_cache::MeshCache;
use crate::preprocess_shader;
use crate::render_pass_command::RenderPassCommand;
use crate::vertex::WgpuVertex;

/// Per pixel: the pick id, and the depth's bits.
const PICK_FORMAT: TextureFormat = TextureFormat::Rg32Uint;
const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
const BYTES_PER_PIXEL: u32 = 8;
/// The tile read around the pointer, in pixels: the pointer and 7 pixels on each side.
const TILE: u32 = 15;
/// A copy's rows must be a multiple of 256 bytes apart.
const ROW_PITCH: u32 = 256;

/// The frame the picking buffer holds.
#[derive(Debug, Copy, Clone)]
struct PickFrame {
    inv_view_proj: Matrix4<f32>,
    camera_position: Vector3<f32>,
    width: u32,
    height: u32,
}

#[derive(Debug)]
pub(crate) struct PickingState {
    pipeline: RenderPipeline,
    /// The ids and the depth, with their size.
    targets: Option<((u32, u32), Texture, TextureView, TextureView)>,
    /// Where a tile is copied to and read from.
    staging: Buffer,
    /// The frame in `targets`, if the last frame rendered one.
    frame: Option<PickFrame>,
}

impl PickingState {
    /// `scene_layout` is the main pass's scene bind group layout, whose objects and camera the
    /// picking pass reads.
    pub(crate) fn new(device: &Device, scene_layout: &BindGroupLayout) -> PickingState {
        let shader = device.create_shader_module(preprocess_shader("pbr/picking.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("carbide_3d_picking_layout"),
            bind_group_layouts: &[scene_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("carbide_3d_picking"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("picking_vs"),
                compilation_options: Default::default(),
                buffers: &[WgpuVertex::desc()],
            },
            // As the main pass: what is picked is what is seen.
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("picking_fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: PICK_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });

        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("carbide_3d_picking_tile"),
            size: (ROW_PITCH * TILE) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        PickingState { pipeline, targets: None, staging, frame: None }
    }

    /// Forget the last frame's buffer: nothing can be picked until a frame renders one.
    pub(crate) fn clear(&mut self) {
        self.frame = None;
    }

    fn ensure_targets(&mut self, device: &Device, width: u32, height: u32) {
        if self.targets.as_ref().is_some_and(|(size, ..)| *size == (width, height)) {
            return;
        }

        let create = |label: &str, format: TextureFormat, usage: wgpu::TextureUsages| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };

        let ids = create("carbide_3d_picking_ids", PICK_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
        let depth = create("carbide_3d_picking_depth", DEPTH_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let ids_view = ids.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());
        self.targets = Some(((width, height), ids, ids_view, depth_view));
    }

    /// Render the picking buffer of the frame `commands` describe, and keep it for `pick`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        (width, height): (u32, u32),
        scene_bind_group: &wgpu::BindGroup,
        commands: &[RenderPassCommand],
        meshes: &MeshCache,
        view_proj: Matrix4<f32>,
        view: Matrix4<f32>,
    ) {
        self.ensure_targets(device, width, height);
        let (_, _, ids_view, depth_view) = self.targets.as_ref().unwrap();

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("carbide_3d_picking"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ids_view,
                resolve_target: None,
                ops: Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(Operations { load: wgpu::LoadOp::Clear(0.0), store: StoreOp::Discard }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);

        // Every mesh: those that aren't pickable still hide what is behind them.
        for command in commands {
            if let RenderPassCommand::DrawMesh { mesh, object, .. } = command {
                let Some(cached) = meshes.get(mesh.id()) else {
                    continue;
                };

                pass.set_vertex_buffer(0, cached.vertex_buffer.slice(..));
                pass.set_index_buffer(cached.index_buffer.slice(..), IndexFormat::Uint32);
                pass.draw_indexed(0..cached.index_count, 0, *object..*object + 1);
            }
        }

        let inv_view = view.invert().unwrap_or(Matrix4::identity());
        self.frame = Some(PickFrame {
            inv_view_proj: view_proj.invert().unwrap_or(Matrix4::identity()),
            camera_position: inv_view.w.truncate(),
            width,
            height,
        });
    }

    /// The nearest pickable pixel within `radius` of `(x, y)` in the last frame's buffer.
    pub(crate) fn pick(&mut self, device: &Device, queue: &Queue, (x, y): (u32, u32), radius: u32) -> Option<PickSample> {
        let frame = self.frame?;
        let (_, ids, _, _) = self.targets.as_ref()?;
        if x >= frame.width || y >= frame.height {
            return None;
        }

        // The tile around the pointer, clamped to the buffer.
        let (tile_width, tile_height) = (TILE.min(frame.width), TILE.min(frame.height));
        let x0 = x.saturating_sub(TILE / 2).min(frame.width - tile_width);
        let y0 = y.saturating_sub(TILE / 2).min(frame.height - tile_height);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("carbide_3d_pick") });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: ids,
                mip_level: 0,
                origin: wgpu::Origin3d { x: x0, y: y0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.staging,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(ROW_PITCH), rows_per_image: Some(tile_height) },
            },
            wgpu::Extent3d { width: tile_width, height: tile_height, depth_or_array_layers: 1 },
        );
        queue.submit(std::iter::once(encoder.finish()));

        let slice = self.staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        // Blocks until the copy, and the frame before it, are done.
        device.poll(wgpu::PollType::Wait).ok()?;

        // The tile's ids and depths, row by row.
        let tile = {
            let data = slice.get_mapped_range();
            let mut tile = vec![(0u32, 0.0f32); (tile_width * tile_height) as usize];
            for row in 0..tile_height {
                for column in 0..tile_width {
                    let at = (row * ROW_PITCH + column * BYTES_PER_PIXEL) as usize;
                    let id = u32::from_le_bytes(data[at..at + 4].try_into().unwrap());
                    let depth = f32::from_bits(u32::from_le_bytes(data[at + 4..at + 8].try_into().unwrap()));
                    tile[(row * tile_width + column) as usize] = (id, depth);
                }
            }
            tile
        };
        self.staging.unmap();

        let at = |column: i32, row: i32| -> Option<(u32, f32)> {
            if column < 0 || row < 0 || column >= tile_width as i32 || row >= tile_height as i32 {
                return None;
            }
            Some(tile[(row as u32 * tile_width + column as u32) as usize])
        };

        // The pointer's pixel, or the nearest pickable one within the radius; nearer the camera
        // (greater depth, reverse-Z) breaks ties.
        let (cx, cy) = ((x - x0) as i32, (y - y0) as i32);
        let radius = radius as i32;
        let mut best: Option<(i32, i32, i32, f32)> = None;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let distance = dx * dx + dy * dy;
                if distance > radius * radius {
                    continue;
                }
                let Some((id, depth)) = at(cx + dx, cy + dy) else {
                    continue;
                };
                if id == 0 {
                    continue;
                }
                let better = match best {
                    None => true,
                    Some((best_distance, _, _, best_depth)) => distance < best_distance || (distance == best_distance && depth > best_depth),
                };
                if better {
                    best = Some((distance, cx + dx, cy + dy, depth));
                }
            }
        }

        let (_, column, row, depth) = best?;
        let pick_id = at(column, row)?.0;

        // A pixel of the tile in world space, from its depth.
        let world = |column: i32, row: i32, depth: f32| -> Vector3<f32> {
            let px = (x0 as i32 + column) as f32 + 0.5;
            let py = (y0 as i32 + row) as f32 + 0.5;
            let ndc = Vector4::new(px / frame.width as f32 * 2.0 - 1.0, 1.0 - py / frame.height as f32 * 2.0, depth, 1.0);
            let world = frame.inv_view_proj * ndc;
            world.truncate() / world.w
        };

        let position = world(column, row, depth);

        // The normal, from the neighbours on the same mesh: the cross product of the steps across
        // and down the surface, turned to face the camera.
        let neighbour = |dc: i32, dr: i32| at(column + dc, row + dr).filter(|(id, _)| *id == pick_id).map(|(_, d)| world(column + dc, row + dr, d));
        let across = neighbour(1, 0).map(|p| p - position).or_else(|| neighbour(-1, 0).map(|p| position - p));
        let down = neighbour(0, 1).map(|p| p - position).or_else(|| neighbour(0, -1).map(|p| position - p));
        let normal = match (across, down) {
            (Some(across), Some(down)) => {
                let normal = across.cross(down);
                if normal.magnitude2() > f32::EPSILON * f32::EPSILON {
                    let normal = normal.normalize();
                    Some(if normal.dot(frame.camera_position - position) < 0.0 { -normal } else { normal })
                } else {
                    None
                }
            }
            _ => None,
        };

        Some(PickSample {
            pick_id,
            position,
            normal,
            distance: (position - frame.camera_position).magnitude(),
        })
    }
}
