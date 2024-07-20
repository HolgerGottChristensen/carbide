
struct Material {
    uv_transform0: mat3x3<f32>,
    // -- 16 --
    uv_transform1: mat3x3<f32>,
    // -- 16 --
    albedo: vec4<f32>, // Base Color
    // -- 16 --
    emissive: vec3<f32>,
    roughness: f32,
    // -- 16 --
    metallic: f32, // When NOT specular glossiness
    reflectance: f32, // When NOT specular glossiness
    clear_coat: f32,
    clear_coat_roughness: f32,
    // -- 16 --
    anisotropy: f32,
    // anisotropic_direction: vec3<f32>
    // specularColor: vec3<f32>; // When specular glossiness
    // glossiness: f32, // When specular glossiness
    ambient_occlusion: f32,
    alpha_cutout: f32,
    flags: u32,
}