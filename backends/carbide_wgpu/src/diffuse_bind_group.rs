use wgpu::{BindGroup, BindGroupLayout, Device, Texture};

use crate::image::Image;

pub type DiffuseBindGroup = BindGroup;

pub fn new_diffuse(device: &Device, image: &Image, glyph_cache_tex: &Texture, atlas_cache_tex: &Texture, layout: &BindGroupLayout) -> DiffuseBindGroup {
    device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&image.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&image.texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&glyph_cache_tex.create_view(&wgpu::TextureViewDescriptor::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&atlas_cache_tex.create_view(&wgpu::TextureViewDescriptor::default())),
                }
            ],
            label: Some("diffuse_bind_group"),
        }
    )
}
