use encase::ShaderType;
use carbide_core::render::matrix::Vector4;

#[derive(Debug, Copy, Clone, ShaderType)]
pub struct WgpuUniforms {
    pub(crate) ambient: Vector4<f32>
}