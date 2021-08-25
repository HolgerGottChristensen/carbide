mod diffuse_bind_group;
mod glyph_cache_command;
mod image;
mod pipeline;
mod render_pass_command;
mod renderer;
mod texture;
mod texture_atlas_command;
mod vertex;
pub mod window;
mod render;
mod filter;

const GLYPH_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
const DEFAULT_IMAGE_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

pub fn init_logger() {
    env_logger::init();
}