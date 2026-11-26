use encase::ShaderType;
use carbide_core::math::Matrix4;

#[derive(Debug, Copy, Clone, ShaderType, PartialEq)]
pub struct WgpuObject {
    pub transform: Matrix4<f32>,
    pub material_index: u32,
    /// 1 when shadows fall on the object, 0 when not.
    pub receives_shadows: u32,
}
