use carbide_core::draw::{InnerImageContext, Texture, TextureFormat, ImageId};

use crate::image::BindGroupExtended;
use crate::application::{DEVICE, QUEUE};
use crate::bind_group_layouts::MAIN_TEXTURE_BIND_GROUP_LAYOUT;
use crate::wgpu_window::{BIND_GROUPS};

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
        BIND_GROUPS.with(|bind_groups| {
            let bind_groups = &mut *bind_groups.borrow_mut();

            let bind_group = create_bind_group(texture);
            bind_groups.insert(id, bind_group);

            true
        })
    }
}

pub fn create_bind_group(
    texture: Texture,
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

    let wgpu_texture = DEVICE.create_texture(&wgpu::TextureDescriptor {
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


    QUEUE.write_texture(
        wgpu::ImageCopyTexture {
            texture: &wgpu_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: Default::default(),
        },
        &texture.data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(texture.bytes_per_row),
            rows_per_image: Some(texture.height),
        },
        size,
    );


    create_bind_group_from_wgpu_texture(&wgpu_texture)
}

pub fn create_bind_group_from_wgpu_texture(wgpu_texture: &wgpu::Texture) -> BindGroupExtended {
    let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = DEVICE.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear, // Change to nearest for pixel images
        min_filter: wgpu::FilterMode::Linear, // Change to nearest for pixel images
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = DEVICE.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &MAIN_TEXTURE_BIND_GROUP_LAYOUT,
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
        label: None,
    });

    BindGroupExtended {
        bind_group,
        width: wgpu_texture.width(),
        height: wgpu_texture.height(),
    }
}