use encase::{ArrayLength, ShaderType};
use carbide_core::render::matrix::{Matrix4, Vector2, Vector3};

#[derive(Debug, Clone, ShaderType)]
pub struct WgpuDirectionalLightBuffer {
    pub count: ArrayLength,
    #[size(runtime)]
    pub array: Vec<WgpuDirectionalLight>,
}

#[derive(Debug, Copy, Clone, ShaderType)]
pub struct WgpuDirectionalLight {
    /// View/Projection of directional light. Shadow rendering uses viewports
    /// so this always outputs [-1, 1] no matter where in the atlast the shadow is.
    pub view_proj: Matrix4<f32>,
    /// Color/intensity of the light
    pub color: Vector3<f32>,
    /// Direction of the light
    pub direction: Vector3<f32>,
    /// 1 / resolution of whole shadow map
    pub inv_resolution: Vector2<f32>,
    /// [0, 1] offset of the shadow map in the atlas.
    pub atlas_offset: Vector2<f32>,
    /// [0, 1] size of the shadow map in the atlas.
    pub atlas_size: Vector2<f32>,
}