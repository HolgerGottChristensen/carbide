use encase::ShaderType;

use carbide_core::render::matrix::Matrix4;

#[derive(Debug, Copy, Clone, ShaderType, PartialEq)]
pub struct WgpuObject {
    pub transform: Matrix4<f32>,
    pub material_index: u32,
}

unsafe impl bytemuck::Zeroable for WgpuObject {}
unsafe impl bytemuck::Pod for WgpuObject {}