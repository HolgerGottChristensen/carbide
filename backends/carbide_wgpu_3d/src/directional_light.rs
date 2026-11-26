use encase::ShaderType;
use carbide_core::math::{Matrix4, SquareMatrix, Vector3, Zero};

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
                shadow_layer: -1,
                direction: Vector3::zero(),
                shadow_bias: 0.0,
                shadow_scale: 1.0,
            });
        }

        WgpuDirectionalLightBuffer { count, array: lights }
    }
}

#[derive(Debug, Copy, Clone, ShaderType)]
pub struct WgpuDirectionalLight {
    /// From world space to the light's shadow map, when it casts shadows.
    pub view_proj: Matrix4<f32>,
    /// The light's linear colour times its intensity in lux.
    pub color: Vector3<f32>,
    /// The light's layer in the shadow maps, or -1 when it casts none.
    pub shadow_layer: i32,
    /// The way the light shines, in world space.
    pub direction: Vector3<f32>,
    /// How far surfaces are moved towards the light along their normal before the shadow test.
    pub shadow_bias: f32,
    /// The part of its layer the light's map fills, across and down: its resolution over the
    /// layer's.
    pub shadow_scale: f32,
}
