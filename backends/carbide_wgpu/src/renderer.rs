use wgpu::{TextureFormat, TextureUsage};

pub fn main_render_tex_desc([width, height]: [u32; 2]) -> wgpu::TextureDescriptor<'static> {
    let depth_or_array_layers = 1;
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers,
    };
    wgpu::TextureDescriptor {
        label: Some("carbide_wgpu_main_render_tex"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC | wgpu::TextureUsage::COPY_DST,
    }
}

// This will return the texture description of the secondary image. This is used as a copy destination
// when for example applying a filter.
pub fn secondary_render_tex_desc([width, height]: [u32; 2]) -> wgpu::TextureDescriptor<'static> {
    let depth_or_array_layers = 1;
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers,
    };
    wgpu::TextureDescriptor {
        label: Some("carbide_wgpu_secondary_render_tex"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    }
}

pub fn atlas_cache_tex_desc([width, height]: [u32; 2]) -> wgpu::TextureDescriptor<'static> {
    let depth_or_array_layers = 1;
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers,
    };
    wgpu::TextureDescriptor {
        label: Some("carbide_wgpu_atlas_cache_texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    }
}
