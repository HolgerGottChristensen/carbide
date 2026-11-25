use encase::ShaderType;
use carbide_core::math::{Vector3, Vector4, Zero};

/// A point or spot light, as the shader reads it. A point light is a spot light whose cone lets
/// all light through: a cone scale of 0 and an offset of 1.
#[derive(Debug, Copy, Clone, ShaderType)]
pub struct WgpuPunctualLight {
    /// The light's position in world space; `w` is 1.
    pub position: Vector4<f32>,
    /// The way a spot light shines in world space; `w` is 0. Unused by point lights.
    pub direction: Vector4<f32>,
    /// The light's linear colour times its intensity in candela.
    pub color: Vector3<f32>,
    /// Where the light has faded to nothing, in metres; 0 for no range.
    pub range: f32,
    /// `1 / max(0.001, cos inner − cos outer)`: how fast the cone fades between its angles.
    pub cone_scale: f32,
    /// `−cos outer · cone_scale`.
    pub cone_offset: f32,
}

#[derive(Debug, Clone, ShaderType)]
pub struct WgpuPunctualLightBuffer {
    /// How many of the lights in `array` are real.
    pub count: u32,
    /// The lights, followed by one unused light when there are none: a storage binding must hold
    /// at least one element of a runtime-sized array.
    #[size(runtime)]
    pub array: Vec<WgpuPunctualLight>,
}

impl WgpuPunctualLightBuffer {
    pub fn new(mut lights: Vec<WgpuPunctualLight>) -> WgpuPunctualLightBuffer {
        let count = lights.len() as u32;

        if lights.is_empty() {
            lights.push(WgpuPunctualLight {
                position: Vector4::zero(),
                direction: Vector4::zero(),
                color: Vector3::zero(),
                range: 0.0,
                cone_scale: 0.0,
                cone_offset: 1.0,
            });
        }

        WgpuPunctualLightBuffer { count, array: lights }
    }
}
