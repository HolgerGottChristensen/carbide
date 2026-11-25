use encase::ShaderType;
use carbide_core::math::{Vector3, Vector4, Zero};

#[derive(Debug, Copy, Clone, ShaderType)]
pub struct WgpuPointLight {
    pub position: Vector4<f32>,
    pub color: Vector3<f32>,
    pub radius: f32,
}

#[derive(Debug, Clone, ShaderType)]
pub struct WgpuPointLightBuffer {
    /// How many of the lights in `array` are real.
    pub count: u32,
    /// The lights, followed by one unused light when there are none: a storage binding must hold
    /// at least one element of a runtime-sized array.
    #[size(runtime)]
    pub array: Vec<WgpuPointLight>,
}

impl WgpuPointLightBuffer {
    pub fn new(mut lights: Vec<WgpuPointLight>) -> WgpuPointLightBuffer {
        let count = lights.len() as u32;

        if lights.is_empty() {
            lights.push(WgpuPointLight {
                position: Vector4::zero(),
                color: Vector3::zero(),
                radius: 0.0,
            });
        }

        WgpuPointLightBuffer { count, array: lights }
    }
}