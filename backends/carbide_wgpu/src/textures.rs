use crate::msaa::Msaa;
use wgpu::{Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView};

pub(crate) fn create_depth_stencil_texture_view(device: &Device, width: u32, height: u32, msaa: Msaa) -> TextureView {
    device.create_texture(&TextureDescriptor {
        label: Some("carbide_depth_stencil_texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: msaa.to_samples(),
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth24PlusStencil8,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    }).create_view(&Default::default())
}

pub(crate) fn create_msaa_texture_view(device: &Device, width: u32, height: u32, msaa: Msaa) -> Option<TextureView> {
    match msaa {
        Msaa::X1 => None,
        Msaa::X4 => Some(
            device.create_texture(&TextureDescriptor {
                label: Some("carbide_msaa_texture"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: msaa.to_samples(),
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
                .create_view(&wgpu::TextureViewDescriptor::default())
        )
    }
}

pub fn create_atlas_cache_texture(device: &Device, width: u32, height: u32) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("carbide_atlas_cache_texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    })
}