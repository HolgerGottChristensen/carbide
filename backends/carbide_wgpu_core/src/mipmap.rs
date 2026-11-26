use std::collections::HashMap;
use wgpu::{BindGroupLayout, Device, Queue, RenderPipeline, Sampler, ShaderModule, TextureFormat};

/// The number of mip levels in a full chain for a texture of `width` × `height`, down to 1×1.
pub fn mip_level_count(width: u32, height: u32) -> u32 {
    32 - width.max(height).max(1).leading_zeros()
}

/// Fills the mip levels of textures from their first level, on the GPU. Each level is drawn from
/// the one above it with linear filtering.
///
/// The textures need `RENDER_ATTACHMENT` and `TEXTURE_BINDING` usage. A pipeline is made for each
/// format the first time a texture of it is filled.
#[derive(Debug)]
pub struct MipmapGenerator {
    shader: ShaderModule,
    layout: BindGroupLayout,
    sampler: Sampler,
    pipelines: HashMap<TextureFormat, RenderPipeline>,
}

impl MipmapGenerator {
    pub fn new(device: &Device) -> MipmapGenerator {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/mipmap.wgsl"));

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("carbide_mipmap_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("carbide_mipmap_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        MipmapGenerator {
            shader,
            layout,
            sampler,
            pipelines: HashMap::new(),
        }
    }

    fn pipeline(&mut self, format: TextureFormat, device: &Device) -> &RenderPipeline {
        let (shader, layout) = (&self.shader, &self.layout);

        self.pipelines.entry(format).or_insert_with(|| {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("carbide_mipmap_pipeline_layout"),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("carbide_mipmap_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        })
    }

    /// Draw every mip level of `texture` after the first from the level above it, read and
    /// written through views of `format`, and submit the work to `queue`.
    pub fn generate(&mut self, texture: &wgpu::Texture, format: TextureFormat, device: &Device, queue: &Queue) {
        let levels = texture.mip_level_count();
        if levels < 2 {
            return;
        }

        let level_view = |level: u32| texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("carbide_mipmap_level"),
            format: Some(format),
            base_mip_level: level,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("carbide_mipmap_encoder"),
        });

        let pipeline = self.pipeline(format, device).clone();

        for level in 1..levels {
            let source = level_view(level - 1);
            let target = level_view(level);

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("carbide_mipmap_bind_group"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&source) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                ],
            });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("carbide_mipmap_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    use super::mip_level_count;

    #[test]
    fn mip_level_counts() {
        assert_eq!(mip_level_count(1, 1), 1);
        assert_eq!(mip_level_count(2, 2), 2);
        assert_eq!(mip_level_count(3, 1), 2);
        assert_eq!(mip_level_count(256, 256), 9);
        assert_eq!(mip_level_count(257, 16), 9);
        assert_eq!(mip_level_count(1024, 512), 11);
        assert_eq!(mip_level_count(0, 0), 1);
    }
}
