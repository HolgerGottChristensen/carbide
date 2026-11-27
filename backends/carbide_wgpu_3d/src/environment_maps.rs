//! Image-based lighting's resources: per environment image, a filtered reflection cube and the
//! spherical harmonics of its diffuse light; and once per device, the DFG lookup table.
//!
//! Everything is drawn on the GPU with render passes, a fullscreen triangle per cube face and
//! level (see `shaders/environment`):
//! 1. The panorama, an equirectangular image from the shared texture store, is drawn into the
//!    first level of a source cube, whose faces are a quarter of the panorama's width, a power of
//!    two from 32 to 512.
//! 2. The source cube's other levels are drawn from the level above, down to 1 × 1.
//! 3. The reflection cube is as large, with levels down to 8 × 8: its first level a copy of the
//!    source's, a mirror; each further level the source convolved with GGX for a perceptual
//!    roughness rising evenly to 1 at the last, as Filament maps roughness to levels.
//! 4. The source's 32 × 32 level is projected onto 9 spherical harmonics, into a 9 × 1 float
//!    texture, which the shading reads.
//!
//! The source cube is dropped once the maps are made.

use std::collections::HashMap;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    AddressMode, BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferUsages, Device, FilterMode, Queue,
    RenderPipeline, Sampler, SamplerBindingType, ShaderStages, Texture, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use carbide_core::draw::ImageId;
use carbide_core::environment::{EnvironmentKey, Exclusive};
use carbide_wgpu_core::WgpuTextureStore;

use crate::preprocess_shader;

/// The format of the cubes and the DFG table.
pub const ENVIRONMENT_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
/// The format of the spherical harmonics: 9 × 1 texels, one per coefficient, RGB.
pub const HARMONICS_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
/// The DFG table's size: N·V across, perceptual roughness down.
pub const DFG_SIZE: u32 = 128;

/// The smallest and largest face of the cubes.
const MIN_FACE: u32 = 32;
const MAX_FACE: u32 = 512;
/// The reflection cube's smallest level.
const SMALLEST_LEVEL: u32 = 8;
/// The size of the source level the harmonics are projected from.
const HARMONICS_LEVEL: u32 = 32;
/// GGX samples per texel of the reflection cube's blurred levels.
const RADIANCE_SAMPLES: u32 = 256;

/// The environment maps made from images, by `ImageId`, and the DFG table, shared by every view
/// the wgpu 3D renderer draws. Maps are made the first time an image is used as an environment,
/// and again when the image is replaced. Nothing is freed yet, as with the texture store.
///
/// Registered in the environment by the renderer's plugin.
#[derive(Debug, Default)]
pub struct EnvironmentMaps {
    maps: HashMap<ImageId, EnvironmentMap>,
    dfg: Option<(Texture, TextureView)>,
    /// Made with the first map or table.
    pipelines: Option<Pipelines>,
    builds: u64,
}

/// An environment image's maps.
#[derive(Debug)]
pub struct EnvironmentMap {
    /// The filtered reflections: level `i` of `levels` for perceptual roughness `i / (levels - 1)`.
    pub radiance: Texture,
    /// A cube view of every level of `radiance`.
    pub radiance_view: TextureView,
    /// The reflection cube's number of levels.
    pub levels: u32,
    /// The 9 spherical-harmonic coefficients of the diffuse light, 9 × 1 texels: evaluated for a
    /// normal, they give the light a white Lambertian surface facing that way reflects.
    pub harmonics: Texture,
    pub harmonics_view: TextureView,
    /// The stored image's version the maps were made from.
    version: u64,
}

impl EnvironmentMaps {
    pub fn new() -> EnvironmentMaps {
        EnvironmentMaps::default()
    }

    /// The maps of the image `id`, made now if they haven't been, or if the image was replaced
    /// since. `None` while the image isn't in the store.
    pub fn prepare(&mut self, id: &ImageId, store: Option<&WgpuTextureStore>, device: &Device, queue: &Queue) -> Option<&EnvironmentMap> {
        let stored = store?.get(id)?;

        if self.maps.get(id).is_none_or(|map| map.version != stored.version) {
            let pipelines = self.pipelines.get_or_insert_with(|| Pipelines::new(device));
            let map = pipelines.build(&stored.srgb_view, stored.width, stored.version, device, queue);
            self.maps.insert(id.clone(), map);
            self.builds += 1;
        }

        self.maps.get(id)
    }

    /// The maps of `id`, if they have been made.
    pub fn get(&self, id: &ImageId) -> Option<&EnvironmentMap> {
        self.maps.get(id)
    }

    /// The DFG lookup table, made on the first call: red is the scale and green the bias of F0.
    pub fn dfg(&mut self, device: &Device, queue: &Queue) -> &TextureView {
        if self.dfg.is_none() {
            let pipelines = self.pipelines.get_or_insert_with(|| Pipelines::new(device));
            self.dfg = Some(pipelines.build_dfg(device, queue));
        }

        &self.dfg.as_ref().unwrap().1
    }

    /// The DFG table's texture, if it has been made.
    pub fn dfg_texture(&self) -> Option<&Texture> {
        self.dfg.as_ref().map(|(texture, _)| texture)
    }

    /// How many times maps have been made, in total.
    pub fn builds(&self) -> u64 {
        self.builds
    }
}

impl EnvironmentKey for EnvironmentMaps {
    type Value = EnvironmentMaps;
    type Access = Exclusive;
}

/// The size of the cubes' faces for a panorama `width` wide.
pub fn face_size(width: u32) -> u32 {
    (width / 4).max(1).next_power_of_two().clamp(MIN_FACE, MAX_FACE)
}

/// The reflection cube's number of levels for faces of `size`: down to 8 × 8.
pub fn radiance_levels(size: u32) -> u32 {
    (size / SMALLEST_LEVEL).max(1).ilog2() + 1
}

#[derive(Debug)]
struct Pipelines {
    sampler: Sampler,
    panorama_layout: BindGroupLayout,
    cube_layout: BindGroupLayout,
    project_layout: BindGroupLayout,
    equirect: RenderPipeline,
    downsample: RenderPipeline,
    radiance: RenderPipeline,
    project: RenderPipeline,
    dfg: RenderPipeline,
}

impl Pipelines {
    fn new(device: &Device) -> Pipelines {
        let texture = |dimension: TextureViewDimension| BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: dimension,
                multisampled: false,
            },
            count: None,
        };
        let sampler = BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        };
        let uniform = BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };

        let layout = |label: &str, entries: &[BindGroupLayoutEntry]| device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some(label), entries });
        let panorama_layout = layout("carbide_3d_environment_panorama_layout", &[texture(TextureViewDimension::D2), sampler, uniform]);
        let cube_layout = layout("carbide_3d_environment_cube_layout", &[texture(TextureViewDimension::Cube), sampler, uniform]);
        let project_layout = layout("carbide_3d_environment_project_layout", &[texture(TextureViewDimension::D2Array)]);

        let pipeline = |shader: &str, entry_point: &str, layouts: &[&BindGroupLayout], format: TextureFormat| {
            let module = device.create_shader_module(preprocess_shader(shader));
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("carbide_3d_environment_pipeline_layout"),
                bind_group_layouts: layouts,
                push_constant_ranges: &[],
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&layout),
                vertex: wgpu::VertexState { module: &module, entry_point: Some("face_vs"), compilation_options: Default::default(), buffers: &[] },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(entry_point),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
                }),
                multiview: None,
                cache: None,
            })
        };

        Pipelines {
            // Linear between texels and levels; repeating across, for the panorama's seam, and
            // clamped down, at its poles. Cubes ignore the address modes.
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("carbide_3d_environment_sampler"),
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Linear,
                ..Default::default()
            }),
            equirect: pipeline("environment/equirect.wgsl", "equirect_fs", &[&panorama_layout], ENVIRONMENT_FORMAT),
            downsample: pipeline("environment/downsample.wgsl", "downsample_fs", &[&cube_layout], ENVIRONMENT_FORMAT),
            radiance: pipeline("environment/radiance.wgsl", "radiance_fs", &[&cube_layout], ENVIRONMENT_FORMAT),
            project: pipeline("environment/project.wgsl", "project_fs", &[&project_layout], HARMONICS_FORMAT),
            dfg: pipeline("environment/dfg.wgsl", "dfg_fs", &[], ENVIRONMENT_FORMAT),
            panorama_layout,
            cube_layout,
            project_layout,
        }
    }

    /// The maps of a panorama `width` wide, read through `panorama`.
    fn build(&self, panorama: &TextureView, width: u32, version: u64, device: &Device, queue: &Queue) -> EnvironmentMap {
        let size = face_size(width);
        let source_levels = size.ilog2() + 1;
        let levels = radiance_levels(size);

        let cube = |label: &str, levels: u32, usage: TextureUsages| device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 6 },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ENVIRONMENT_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | usage,
            view_formats: &[],
        });
        let source = cube("carbide_3d_environment_source", source_levels, TextureUsages::COPY_SRC);
        let radiance = cube("carbide_3d_environment_radiance", levels, TextureUsages::COPY_DST | TextureUsages::COPY_SRC);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("carbide_3d_environment_encoder") });

        // 1. The panorama into the source's first level, read at the level whose texels are about
        // the size of the face's.
        let panorama_level = (width as f32 / (4 * size) as f32).max(1.0).log2();
        let bind_group = self.bind_group(&self.panorama_layout, panorama, Some(&[size as f32, panorama_level]), device);
        draw_faces(&mut encoder, &self.equirect, &bind_group, &source, 0);

        // 2. Each further level of the source from the one above.
        for level in 1..source_levels {
            let above = source.create_view(&TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                base_mip_level: level - 1,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let bind_group = self.bind_group(&self.cube_layout, &above, Some(&[(size >> level) as f32]), device);
            draw_faces(&mut encoder, &self.downsample, &bind_group, &source, level);
        }

        // 3. The reflections: a mirror, then GGX for rising roughness, from the whole source.
        let face = wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 6 };
        encoder.copy_texture_to_texture(source.as_image_copy(), radiance.as_image_copy(), face);
        let source_view = source.create_view(&TextureViewDescriptor { dimension: Some(TextureViewDimension::Cube), ..Default::default() });
        for level in 1..levels {
            let roughness = level as f32 / (levels - 1) as f32;
            // The sample count is a `u32` in the shader: its bits, carried as an `f32`.
            let params = [(size >> level) as f32, roughness, f32::from_bits(RADIANCE_SAMPLES), size as f32];
            let bind_group = self.bind_group(&self.cube_layout, &source_view, Some(&params), device);
            draw_faces(&mut encoder, &self.radiance, &bind_group, &radiance, level);
        }

        // 4. The harmonics, from the source's 32 × 32 level.
        let harmonics = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("carbide_3d_environment_harmonics"),
            size: wgpu::Extent3d { width: 9, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HARMONICS_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let harmonics_view = harmonics.create_view(&Default::default());
        let small = source.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2Array),
            base_mip_level: (size / HARMONICS_LEVEL).ilog2(),
            mip_level_count: Some(1),
            ..Default::default()
        });
        let bind_group = self.bind_group(&self.project_layout, &small, None, device);
        draw(&mut encoder, &self.project, Some(&bind_group), &harmonics_view, 0..1);

        queue.submit(Some(encoder.finish()));

        let radiance_view = radiance.create_view(&TextureViewDescriptor {
            label: Some("carbide_3d_environment_radiance_view"),
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });

        EnvironmentMap { radiance, radiance_view, levels, harmonics, harmonics_view, version }
    }

    fn build_dfg(&self, device: &Device, queue: &Queue) -> (Texture, TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("carbide_3d_dfg"),
            size: wgpu::Extent3d { width: DFG_SIZE, height: DFG_SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ENVIRONMENT_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("carbide_3d_dfg_encoder") });
        draw(&mut encoder, &self.dfg, None, &view, 0..1);
        queue.submit(Some(encoder.finish()));

        (texture, view)
    }

    /// A bind group of `texture`, and for layouts that take them, the sampler and `params` as a
    /// uniform buffer.
    fn bind_group(&self, layout: &BindGroupLayout, texture: &TextureView, params: Option<&[f32]>, device: &Device) -> wgpu::BindGroup {
        let buffer = params.map(|params| {
            // Uniform buffers are at least 16 bytes.
            let mut padded = params.to_vec();
            padded.resize(params.len().next_multiple_of(4), 0.0);
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("carbide_3d_environment_params"),
                contents: bytemuck::cast_slice(&padded),
                usage: BufferUsages::UNIFORM,
            })
        });

        let mut entries = vec![wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(texture) }];
        if let Some(buffer) = &buffer {
            entries.push(wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) });
            entries.push(wgpu::BindGroupEntry { binding: 2, resource: buffer.as_entire_binding() });
        }

        device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("carbide_3d_environment_bind_group"), layout, entries: &entries })
    }
}

/// Draw the six faces of `level` of `cube` with `pipeline`, each face its own pass.
fn draw_faces(encoder: &mut wgpu::CommandEncoder, pipeline: &RenderPipeline, bind_group: &wgpu::BindGroup, cube: &Texture, level: u32) {
    for face in 0..6 {
        let view = cube.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2),
            base_mip_level: level,
            mip_level_count: Some(1),
            base_array_layer: face,
            array_layer_count: Some(1),
            ..Default::default()
        });
        draw(encoder, pipeline, Some(bind_group), &view, face..face + 1);
    }
}

/// Draw a fullscreen triangle into `target`, the instance being the face.
fn draw(encoder: &mut wgpu::CommandEncoder, pipeline: &RenderPipeline, bind_group: Option<&wgpu::BindGroup>, target: &TextureView, face: std::ops::Range<u32>) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("carbide_3d_environment_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    pass.set_pipeline(pipeline);
    if let Some(bind_group) = bind_group {
        pass.set_bind_group(0, bind_group, &[]);
    }
    pass.draw(0..3, face);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_are_a_quarter_of_the_panorama_clamped() {
        assert_eq!(face_size(1024), 256);
        assert_eq!(face_size(1000), 256);
        assert_eq!(face_size(8), 32);
        assert_eq!(face_size(8192), 512);
    }

    #[test]
    fn reflection_levels_go_down_to_eight() {
        assert_eq!(radiance_levels(256), 6);
        assert_eq!(radiance_levels(32), 3);
        assert_eq!(radiance_levels(512), 7);
    }
}
