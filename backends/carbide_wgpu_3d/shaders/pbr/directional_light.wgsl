
struct DirectionalLight {
    /// View/Projection of directional light. Shadow rendering uses viewports
    /// so this always outputs [-1, 1] no matter where in the atlast the shadow is.
    view_proj: mat4x4<f32>,
    /// Color/intensity of the light
    color: vec3<f32>,
    /// Direction of the light
    direction: vec3<f32>,
    /// 1 / resolution of whole shadow map
    inv_resolution: vec2<f32>,
    /// [0, 1] offset of the shadow map in the atlas.
    offset: vec2<f32>,
    /// [0, 1] size of the shadow map in the atlas.
    size: vec2<f32>,
}

struct DirectionalLights {
    count: u32,
    data: array<DirectionalLight>,
}