use carbide_core::draw::{Texture, TextureFormat};
use wgpu::{BindGroup, BindGroupLayout, Device, Queue, TextureView};

pub use carbide_wgpu_core::WgpuImageContext;

/// Upload `texture` and make a 2D bind group for it. Only used for the renderer's own default
/// texture; images go through the shared `WgpuTextureStore` store.
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
        TextureFormat::RGBA16F => wgpu::TextureFormat::Rgba16Float,
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
        texture.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(texture.bytes_per_row),
            rows_per_image: Some(texture.height),
        },
        size,
    );

    let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

    create_bind_group_from_view(&view, wgpu_texture.width(), wgpu_texture.height(), 0, device, main_texture_bind_group_layout)
}

#[derive(Debug)]
pub struct BindGroupExtended {
    pub bind_group: BindGroup,
    pub width: u32,
    pub height: u32,
    /// The `StoredTexture::version` the bind group was made from, or 0 for the default texture.
    pub version: u64,
}

/// Make a 2D bind group for `view`: the texture and a clamping, linear sampler.
pub fn create_bind_group_from_view(view: &TextureView, width: u32, height: u32, version: u64, device: &Device, main_texture_bind_group_layout: &BindGroupLayout) -> BindGroupExtended {

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear, // Change to nearest for pixel images
        min_filter: wgpu::FilterMode::Linear, // Change to nearest for pixel images
        // Blends between mip levels, for images drawn smaller than their size.
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: main_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
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
        width,
        height,
        version,
    }
}