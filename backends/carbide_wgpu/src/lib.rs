mod texture;
mod image;
mod render_pass_command;
mod glyph_cache_command;
mod diffuse_bind_group;
pub mod window;
mod renderer;
mod texture_atlas_command;
mod vertex;
mod pipeline;

const GLYPH_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
const DEFAULT_IMAGE_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
