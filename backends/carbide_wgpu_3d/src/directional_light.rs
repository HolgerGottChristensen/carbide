use encase::ShaderType;
use carbide_core::math::{Matrix4, SquareMatrix, Vector2, Vector3, Zero};

#[derive(Debug, Clone, ShaderType)]
pub struct WgpuDirectionalLightBuffer {
    /// How many of the lights in `array` are real.
    pub count: u32,
    /// The lights, followed by one unused light when there are none: a storage binding must hold
    /// at least one element of a runtime-sized array.
    #[size(runtime)]
    pub array: Vec<WgpuDirectionalLight>,
}

impl WgpuDirectionalLightBuffer {
    pub fn new(mut lights: Vec<WgpuDirectionalLight>) -> WgpuDirectionalLightBuffer {
        let count = lights.len() as u32;

        if lights.is_empty() {
            lights.push(WgpuDirectionalLight {
                view_proj: Matrix4::identity(),
                color: Vector3::zero(),
                direction: Vector3::zero(),
                inv_resolution: Vector2::zero(),
                atlas_offset: Vector2::zero(),
                atlas_size: Vector2::zero(),
            });
        }

        WgpuDirectionalLightBuffer { count, array: lights }
    }
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