use encase::ShaderType;
use carbide_core::math::{Matrix4, SquareMatrix, Vector3, Vector4, Zero};

/// A point or spot light, as the shader reads it. A point light is a spot light whose cone lets
/// all light through: a cone scale of 0 and an offset of 1.
#[derive(Debug, Copy, Clone, ShaderType)]
pub struct WgpuPunctualLight {
    /// From world space to the light's shadow map, when it casts shadows.
    pub view_proj: Matrix4<f32>,
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
    /// The light's layer in the shadow maps, or -1 when it casts none.
    pub shadow_layer: i32,
    /// How far surfaces are moved towards the light along their normal before the shadow test.
    pub shadow_bias: f32,
    /// The part of its layer the light's map fills, across and down: its resolution over the
    /// layer's.
    pub shadow_scale: f32,
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
                view_proj: Matrix4::identity(),
                position: Vector4::zero(),
                direction: Vector4::zero(),
                color: Vector3::zero(),
                range: 0.0,
                cone_scale: 0.0,
                cone_offset: 1.0,
                shadow_layer: -1,
                shadow_bias: 0.0,
                shadow_scale: 1.0,
            });
        }

        WgpuPunctualLightBuffer { count, array: lights }
    }
}
