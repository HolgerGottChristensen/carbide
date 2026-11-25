use encase::ShaderType;
use carbide_core::math::Vector4;

#[derive(Debug, Copy, Clone, ShaderType)]
pub struct WgpuUniforms {
    /// The hemisphere's light from above, linear, times its intensity.
    pub(crate) sky: Vector4<f32>,
    /// The hemisphere's light from below, linear, times its intensity.
    pub(crate) ground: Vector4<f32>,
    /// What lit colours are multiplied by before tone mapping.
    pub(crate) exposure: f32,
    /// 0: PBR Neutral, 1: ACES, 2: none (clamp).
    pub(crate) tonemapping: u32,
}
