
struct PixelData {
    albedo: vec4<f32>,
    diffuse_color: vec3<f32>,
    roughness: f32,
    normal: vec3<f32>,
    metallic: f32,
    f0: vec3<f32>,
    perceptual_roughness: f32,
    emissive: vec3<f32>,
    reflectance: f32,
    clear_coat: f32,
    clear_coat_roughness: f32,
    clear_coat_perceptual_roughness: f32,
    anisotropy: f32,
    ambient_occlusion: f32,
    material_flags: u32,
}