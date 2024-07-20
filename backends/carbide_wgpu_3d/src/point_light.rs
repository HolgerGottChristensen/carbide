use encase::{ArrayLength, ShaderType};
use carbide_core::render::matrix::{Vector3, Vector4};

#[derive(Debug, Copy, Clone, ShaderType)]
pub struct WgpuPointLight {
    pub position: Vector4<f32>,
    pub color: Vector3<f32>,
    pub radius: f32,
}

#[derive(Debug, Clone, ShaderType)]
pub struct WgpuPointLightBuffer {
    pub count: ArrayLength,
    #[size(runtime)]
    pub array: Vec<WgpuPointLight>,
}