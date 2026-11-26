/// How a light casts shadows, as a render context is told: the light casts shadows only when it
/// is given one of these.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LightShadow {
    /// The shadow map's width and height, in texels.
    pub resolution: u32,
    /// How far, in metres, a surface is moved towards the light along its normal before it is
    /// tested, so it doesn't shadow itself ("shadow acne"). Too much detaches shadows from the
    /// objects casting them.
    pub bias: f32,
    /// For directional lights: the distance in front of the camera, in metres, that the shadow
    /// map covers. Unused by spot lights, whose map covers their cone.
    pub distance: f32,
}

impl LightShadow {
    /// 2048 texels, a bias of 2 cm, and 10 metres in front of the camera.
    pub const DEFAULT: LightShadow = LightShadow {
        resolution: 2048,
        bias: 0.02,
        distance: 10.0,
    };
}

impl Default for LightShadow {
    fn default() -> Self {
        LightShadow::DEFAULT
    }
}
