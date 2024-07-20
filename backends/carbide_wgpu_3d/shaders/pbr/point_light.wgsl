
struct PointLight {
    /// The position of the light in world space.
    position: vec4<f32>,
    // Color/intensity of the light.
    color: vec3<f32>,
    /// The radius of the light.
    radius: f32,
}

struct PointLights {
    count: u32,
    data: array<PointLight>,
}