use wgpu::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

pub fn atlas_cache_tex_desc(width: u32, height: u32) -> TextureDescriptor<'static> {
    let depth_or_array_layers = 1;
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers,
    };

    TextureDescriptor {
        label: Some("carbide_wgpu_atlas_cache_texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    }
}
