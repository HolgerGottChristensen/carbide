
fn mat3_inv_scale_squared(transform: mat3x3<f32>) -> vec3<f32> {
    return vec3<f32>(
        1.0 / dot(transform[0].xyz, transform[0].xyz),
        1.0 / dot(transform[1].xyz, transform[1].xyz),
        1.0 / dot(transform[2].xyz, transform[2].xyz)
    );
}

fn perceptual_roughness_to_roughness(perceptual_roughness: f32) -> f32 {
    return perceptual_roughness * perceptual_roughness;
}

fn compute_diffuse_color(base_color: vec3<f32>, metallic: f32) -> vec3<f32> {
    return base_color * (1.0 - metallic);
}

fn compute_f0(base_color: vec3<f32>, metallic: f32, reflectance: f32) -> vec3<f32> {
    return base_color * metallic + (reflectance * (1.0 - metallic));
}

fn compute_dielectric_f0(reflectance: f32) -> f32 {
    return 0.16 * reflectance * reflectance;
}

fn pow5(val: f32) -> f32 {
    let val2 = val * val;
    return val2 * val2 * val;
}