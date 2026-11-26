struct DirectionalLight {
    /// From world space to the light's shadow map, when it casts shadows.
    view_proj: mat4x4<f32>,
    /// The light's linear colour times its intensity in lux.
    color: vec3<f32>,
    /// The light's layer in the shadow maps, or -1 when it casts none.
    shadow_layer: i32,
    /// The way the light shines, in world space.
    direction: vec3<f32>,
    /// How far surfaces are moved towards the light along their normal before the shadow test.
    shadow_bias: f32,
    /// The part of its layer the light's map fills: its resolution over the layer's.
    shadow_scale: f32,
}

struct DirectionalLights {
    count: u32,
    data: array<DirectionalLight>,
}
