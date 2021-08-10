use wgpu::TextureFormat;

use crate::{DEFAULT_IMAGE_TEX_FORMAT, GLYPH_TEX_FORMAT};

pub fn glyph_cache_tex_desc([width, height]: [u32; 2]) -> wgpu::TextureDescriptor<'static> {
    let depth = 1;
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth,
    };
    wgpu::TextureDescriptor {
        label: Some("carbide_wgpu_glyph_cache_texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: GLYPH_TEX_FORMAT,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    }
}

pub fn atlas_cache_tex_desc([width, height]: [u32; 2]) -> wgpu::TextureDescriptor<'static> {
    let depth = 1;
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth,
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
