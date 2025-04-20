use carbide_core::draw::{ImageContext, ImageId, Texture, TextureFormat};
use wgpu::{BindGroup, BindGroupLayout, Device, Queue};
use carbide_core::environment::Environment;
use crate::wgpu_context::WgpuContext;

pub struct WGPUImageContext;

impl ImageContext for WGPUImageContext {
    fn texture_exist(&self, id: &ImageId, env: &mut Environment) -> bool {
        let wgpu_context = env.get_mut::<WgpuContext>().unwrap();
        wgpu_context.bind_groups.contains_key(id)
    }

    fn texture_dimensions(&self, id: &ImageId, env: &mut Environment) -> Option<(u32, u32)> {
        let wgpu_context = env.get_mut::<WgpuContext>().unwrap();

        wgpu_context.bind_groups.get(id)
            .map(|group| {
                (group.width, group.height)
            })
    }

    fn update_texture(&mut self, id: ImageId, texture: Texture, env: &mut Environment) -> bool {
        let wgpu_context = env.get_mut::<WgpuContext>().unwrap();

        let bind_group = create_bind_group(&wgpu_context.device, &wgpu_context.queue, texture, &wgpu_context.texture_bind_group_layout);
        wgpu_context.bind_groups.insert(id, bind_group);

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