struct Uniforms {
    sky: vec4<f32>,
    ground: vec4<f32>,
    exposure: f32,
    tonemapping: u32,
    // 1 when the scene has an environment (`pbr/environment.wgsl`), lighting it in place of the
    // hemisphere.
    environment: u32,
    environment_intensity: f32,
    // The environment's reflection levels.
    environment_levels: f32,
    // The cosine and sine of the environment's rotation, which `environment_direction` turns
    // world directions back by.
    environment_rotation: vec2<f32>,
}
