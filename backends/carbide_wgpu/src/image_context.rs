use std::rc::Rc;
use carbide_core::draw::{Dimension, ImageContext, ImageFormat, ImageId, ImageMetrics, Texture, TextureFormat};
use wgpu::{BindGroup, BindGroupLayout, Device, Queue};
use carbide_core::environment::Environment;
use carbide_core::render::{RenderInstruction, RenderInstructionCache};
use crate::wgpu_context::WgpuContext;

pub struct WGPUImageContext;

impl ImageContext for WGPUImageContext {
    fn exist(&self, id: &ImageId, env: &mut Environment) -> bool {
        match id.format() {
            ImageFormat::Unknown => false,
            ImageFormat::Svg => {
                let cache = env.get::<RenderInstructionCache>().unwrap();
                cache.contains_key(id)
            }
            _ => {
                let wgpu_context = env.get::<WgpuContext>().unwrap();
                wgpu_context.bind_groups.contains_key(id)
            }
        }
    }

    fn metrics(&self, id: &ImageId, env: &mut Environment) -> ImageMetrics {
        match id.format() {
            ImageFormat::Unknown => ImageMetrics::Unknown,
            ImageFormat::Svg => {
                let cache = env.get::<RenderInstructionCache>().unwrap();
                cache
                    .get(id)
                    .map(|vector| ImageMetrics::Vector { dimension: vector.0 })
                    .unwrap_or(ImageMetrics::Unknown)
            }
            _ => {
                let wgpu_context = env.get::<WgpuContext>().unwrap();
                wgpu_context.bind_groups.get(id)
                    .map(|group| {
                        ImageMetrics::Raster { width: group.width, height: group.height}
                    }).unwrap_or(ImageMetrics::Unknown)
            }
        }
    }

    fn update_texture(&mut self, id: &ImageId, texture: Texture, env: &mut Environment) -> bool {
        let wgpu_context = env.get_mut::<WgpuContext>().unwrap();

        let bind_group = create_bind_group(&wgpu_context.device, &wgpu_context.queue, texture, &wgpu_context.texture_bind_group_layout);
        wgpu_context.bind_groups.insert(id.clone(), bind_group);

        true
    }

    fn update_vector(&mut self, id: &ImageId, description: Vec<RenderInstruction>, size: Dimension, env: &mut Environment) -> bool {
        let cache = env.get_mut::<RenderInstructionCache>().unwrap();

        cache.insert(id.clone(), Rc::new((size, description)));

        true
    }
}

pub fn create_bind_group(
    device: &Device,
    queue: &Queue,
    texture: Texture,
    main_texture_bind_group_layout: &BindGroupLayout
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
        wgpu::TexelCopyTextureInfo {
            texture: &wgpu_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: Default::default(),
        },
        &texture.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(texture.bytes_per_row),
            rows_per_image: Some(texture.height),
        },
        size,
    );

    create_bind_group_from_wgpu_texture(&wgpu_texture, device, main_texture_bind_group_layout)
}

#[derive(Debug)]
pub struct BindGroupExtended {
    pub bind_group: BindGroup,
    pub width: u32,
    pub height: u32,
}

pub fn create_bind_group_from_wgpu_texture(wgpu_texture: &wgpu::Texture, device: &Device, main_texture_bind_group_layout: &BindGroupLayout) -> BindGroupExtended {
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
        layout: main_texture_bind_group_layout,
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
        label: Some("carbide_texture_bind_group"),
    });

    BindGroupExtended {
        bind_group,
        width: wgpu_texture.width(),
        height: wgpu_texture.height(),
    }
}