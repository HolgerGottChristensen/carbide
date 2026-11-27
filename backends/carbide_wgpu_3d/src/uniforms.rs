use encase::ShaderType;
use carbide_core::math::{Vector2, Vector4};

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
    /// 1 when the scene has an environment, lighting it in place of the hemisphere.
    pub(crate) environment: u32,
    /// What the environment's light is multiplied by.
    pub(crate) environment_intensity: f32,
    /// The environment's reflection levels.
    pub(crate) environment_levels: f32,
    /// The cosine and sine of the environment's rotation; the shader turns world directions back
    /// by it to read the environment.
    pub(crate) environment_rotation: Vector2<f32>,
    /// The reflection level the background reads, when it shows the environment.
    pub(crate) background_level: f32,
}
