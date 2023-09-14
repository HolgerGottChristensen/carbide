use std::collections::HashMap;
use std::num::NonZeroU32;
use wgpu::{BindGroup, BindGroupLayout, Device, Queue, Texture as WGPUTexture};
use carbide_core::draw::image::ImageId;
use carbide_core::draw::{InnerImageContext, Texture, TextureFormat};
use crate::diffuse_bind_group::DiffuseBindGroup;
use crate::image::BindGroupExtended;
use crate::wgpu_window::{ATLAS_CACHE_TEXTURE, BIND_GROUPS, DEVICE_QUEUE, MAIN_TEXTURE_BIND_GROUP_LAYOUT};

pub struct WGPUImageContext;

impl InnerImageContext for WGPUImageContext {
    fn texture_exist(&self, id: &ImageId) -> bool {
        BIND_GROUPS.with(|bind_groups| {
            let bind_groups = &mut *bind_groups.borrow_mut();
            bind_groups.contains_key(id)
        })
    }

    fn texture_dimensions(&self, id: &ImageId) -> Option<(u32, u32)> {
        BIND_GROUPS.with(|bind_groups| {
            let bind_groups = &mut *bind_groups.borrow_mut();
            bind_groups.get(id)
                .map(|group| {
                    (group.width, group.height)
                })
        })
    }

    fn update_texture(&mut self, id: ImageId, texture: Texture) -> bool {
        DEVICE_QUEUE.with(|(device, queue)| {
            BIND_GROUPS.with(|bind_groups| {
                ATLAS_CACHE_TEXTURE.with(|atlas_cache_tex| {
                    MAIN_TEXTURE_BIND_GROUP_LAYOUT.with(|texture_bind_group_layout| {
                        let bind_groups = &mut *bind_groups.borrow_mut();

                        //println!("Update image called");
                        let bind_group = create_bind_group(texture, device, queue, atlas_cache_tex, texture_bind_group_layout);
                        bind_groups.insert(id, bind_group);

                        true
                    })
                })
            })
        })
    }
}

pub fn create_bind_group(
    texture: Texture,
    device: &Device,
    queue: &Queue,
    atlas_cache: &WGPUTexture,
    layout: &BindGroupLayout
) -> BindGroupExtended {
    let width = texture.width;
    let height = texture.height;

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let format = match texture.format {
        TextureFormat::RGBA8 => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::BGRA8 => wgpu::TextureFormat::Bgra8UnormSrgb,
    };

    let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    //println!("len: {}", texture.data.len());
    //println!("bytes_per_row: {}", texture.bytes_per_row);
    //println!("height: {}", texture.height);
    //println!("size: {}", texture.height * texture.bytes_per_row);


    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &wgpu_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: Default::default(),
        },
        &texture.data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(texture.bytes_per_row),
            rows_per_image: NonZeroU32::new(texture.height),
        },
        size,
    );

    let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear, // Change to nearest for pixel images
        min_filter: wgpu::FilterMode::Linear, // Change to nearest for pixel images
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(
                    &atlas_cache.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
        ],
        label: None,
    });

    BindGroupExtended {
        bind_group,
        width,
        height,
    }
}