use wgpu::{BindGroup, Texture, TextureDescriptor, TextureFormat, TextureUsages, TextureView};

use crate::wgpu_window::{DEVICE_QUEUE, MAIN_SAMPLER, MAIN_TEXTURE_BIND_GROUP_LAYOUT};

pub struct RenderTarget {
    pub(crate) texture: Texture,
    pub(crate) view: TextureView,
    pub(crate) bind_group: BindGroup,
}

impl RenderTarget {
    pub(crate) fn new(width: u32, height: u32) -> RenderTarget {
        let depth_or_array_layers = 1;

        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };
        let descriptor = TextureDescriptor {
            label: Some("carbide_render_target"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let texture = DEVICE_QUEUE.with(|(device, _)| {
            device.create_texture(&descriptor)
        });

        let view = texture.create_view(&Default::default());

        let bind_group = DEVICE_QUEUE.with(|(device, _)| {
            MAIN_TEXTURE_BIND_GROUP_LAYOUT.with(|layout| {
                MAIN_SAMPLER.with(|sampler| {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&sampler),
                            },
                        ],
                        label: Some("diffuse_bind_group"),
                    })
                })
            })
        });

        RenderTarget {
            texture,
            view,
            bind_group,
        }
    }
}