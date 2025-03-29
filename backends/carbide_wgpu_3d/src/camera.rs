use encase::ShaderType;
use carbide_core::math::Matrix4;

#[derive(Debug, Copy, Clone, ShaderType, PartialEq)]
pub struct WgpuCamera {
    pub(crate) view: Matrix4<f32>,
    pub(crate) view_proj: Matrix4<f32>,
    pub(crate) orig_view: Matrix4<f32>,
    pub(crate) inv_view: Matrix4<f32>,
    pub(crate) aspect_ratio: f32,
}

unsafe impl bytemuck::Zeroable for WgpuCamera {}
unsafe impl bytemuck::Pod for WgpuCamera {}