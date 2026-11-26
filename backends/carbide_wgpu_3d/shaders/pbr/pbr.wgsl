//!include pbr/directional_light.wgsl
//!include pbr/point_light.wgsl
//!include pbr/material.wgsl
//!include pbr/pixel_data.wgsl
//!include pbr/camera.wgsl
//!include pbr/object.wgsl
//!include pbr/surface_shading.wgsl
//!include pbr/util.wgsl
//!include pbr/normal.wgsl
//!include pbr/albedo.wgsl
//!include pbr/aomr.wgsl
//!include pbr/reflectance.wgsl
//!include pbr/clear_coat.wgsl
//!include pbr/uniforms.wgsl
//!include pbr/tonemapping.wgsl
//!include pbr/shadow_lookup.wgsl

const PI = 3.14159265358979323846;
const PI_2 = 1.570796327;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) tangent: vec3<f32>,
    @location(2) texture_coords_0: vec2<f32>,
    @location(3) texture_coords_1: vec2<f32>,
    @location(4) color_0: vec4<f32>,
    @location(5) color_1: vec4<f32>,
    @location(6) @interpolate(flat) material: u32,
    @location(7) view_position: vec4<f32>,
    @location(8) world_position: vec3<f32>,
    @location(9) @interpolate(flat) receives_shadows: u32,
}

@group(0) @binding(0)
var<storage> materials: array<Material>;
@group(0) @binding(1)
var<storage> objects: array<Object>;
@group(0) @binding(2)
var<storage> camera: Camera;

@group(1) @binding(0)
var<storage> directional_lights: DirectionalLights;
@group(1) @binding(1)
var<storage> punctual_lights: PunctualLights;
@group(1) @binding(2)
var primary_sampler: sampler;
@group(1) @binding(3)
var<storage> uniforms: Uniforms;
@group(1) @binding(4)
var shadow_maps: texture_depth_2d_array;
@group(1) @binding(5)
var shadow_sampler: sampler_comparison;
@group(1) @binding(6)
var nearest_sampler: sampler;

// The material's textures, one per slot. A slot whose bit in `texture_enable` is not set holds a
// fallback texture, and is not sampled.
@group(2) @binding(0)
var albedo_tex: texture_2d<f32>;
@group(2) @binding(1)
var normal_tex: texture_2d<f32>;
@group(2) @binding(2)
var ambient_occlusion_tex: texture_2d<f32>;
@group(2) @binding(3)
var metallic_tex: texture_2d<f32>;
@group(2) @binding(4)
var roughness_tex: texture_2d<f32>;
@group(2) @binding(5)
var clear_coat_tex: texture_2d<f32>;
@group(2) @binding(6)
var clear_coat_roughness_tex: texture_2d<f32>;

// Sample `tex` with the material's sampler: nearest when the material asks for it (NEAREST),
// otherwise linear and anisotropic. The gradients come from the unbranched UVs, so sampling inside
// a branch picks the right mip level.
fn sample_material_texture(tex: texture_2d<f32>, material: Material, coords: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> vec4<f32> {
    if ((material.flags & 0x4000u) != 0u) {
        return textureSampleGrad(tex, nearest_sampler, coords, ddx, ddy);
    }
    return textureSampleGrad(tex, primary_sampler, coords, ddx, ddy);
}

@vertex
fn main_vs(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) texture_coords_0: vec2<f32>,
    @location(4) texture_coords_1: vec2<f32>,
    @location(5) color_0: vec4<f32>,
    @location(6) color_1: vec4<f32>,
    @builtin(instance_index) object_index: u32,
) -> VertexOutput {
    let object = objects[object_index];

    let model_view = camera.view * object.transform;
    let model_view_proj = camera.view_proj * object.transform;

    let mv_mat3 = mat3x3<f32>(model_view[0].xyz, model_view[1].xyz, model_view[2].xyz);

    let inv_scale_sq = mat3_inv_scale_squared(mv_mat3);

    var out: VertexOutput;
    out.position = model_view_proj * vec4<f32>(position, 1.0);
    out.view_position = model_view * vec4<f32>(position, 1.0);
    out.normal = normalize(mv_mat3 * (inv_scale_sq * normal));
    out.tangent = normalize(mv_mat3 * (inv_scale_sq * tangent));
    //out.normal = normalize((inv_scale_sq * normal));
    //out.tangent = normalize((inv_scale_sq * tangent));
    //out.normal = normalize((mv_mat3 * normal));
    //out.tangent = normalize((mv_mat3 * tangent));
    //out.normal = normalize(normal);
    //out.tangent = normalize(tangent);
    out.texture_coords_0 = texture_coords_0;
    out.texture_coords_1 = texture_coords_1;
    out.color_0 = color_0;
    out.color_1 = color_1;
    out.material = object.material_index;
    out.world_position = (object.transform * vec4<f32>(position, 1.0)).xyz;
    out.receives_shadows = object.receives_shadows;

    return out;
}

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let material = materials[in.material];
    var pixel: PixelData;

    let coords = (material.uv_transform0 * vec3<f32>(in.texture_coords_0, 1.0)).xy;
    let uvdx = dpdx(coords);
    let uvdy = dpdy(coords);

    pixel.albedo = get_albedo(material, in, coords, uvdx, uvdy);

    if (pixel.albedo.a < material.alpha_cutout) {
        discard;
    }

    // Unlit materials show their colour as it is: no lighting, exposure or tone mapping.
    if ((material.flags & 0x2000u) != 0u) {
        return pixel.albedo;
    }
    pixel.normal = get_normal(material, primary_sampler, in, coords, uvdx, uvdy);

    let aomr = get_aomr(material);
    pixel.ambient_occlusion = aomr.x;
    pixel.metallic = aomr.y;

    pixel.diffuse_color = compute_diffuse_color(pixel.albedo.xyz, pixel.metallic);

    var perceptual_roughness = aomr.z;

    // Assumes an interface from air to an IOR of 1.5 for dielectrics
    let reflectance = compute_dielectric_f0(get_reflectance(material));
    pixel.f0 = compute_f0(pixel.albedo.rgb, pixel.metallic, reflectance);


    let clear_coat = get_clear_coat(material);
    pixel.clear_coat = clear_coat.x;
    pixel.clear_coat_perceptual_roughness = clear_coat.y;
    pixel.clear_coat_roughness = clear_coat.z;

    if (pixel.clear_coat != 0.0) {
        let base_perceptual_roughness = max(perceptual_roughness, pixel.clear_coat_perceptual_roughness);
        perceptual_roughness = mix(perceptual_roughness, base_perceptual_roughness, pixel.clear_coat);
    }
    // https://github.com/google/filament/blob/3728f0660395d04d5735d85831a43480501f1c63/shaders/src/common_material.fs#L4
    pixel.perceptual_roughness = clamp(perceptual_roughness, 0.045, 1.0);
    pixel.roughness = perceptual_roughness_to_roughness(pixel.perceptual_roughness);

    //vec3 direction = material.anisotropyDirection;
    pixel.anisotropy = material.anisotropy;
    //pixel.anisotropicT = normalize(shading_tangentToWorld * direction);
    //pixel.anisotropicB = normalize(cross(getWorldGeometricNormalVector(), pixel.anisotropicT));


    var color = vec3<f32>(0.0);




    // View vector
    let v = -normalize(in.view_position.xyz);

    let view_mat3 = mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz);

    // For the shadow lookups: the surface's geometric normal in world space, and whether shadows
    // fall on it.
    let world_normal = normalize((camera.inv_view * vec4<f32>(in.normal, 0.0)).xyz);
    let receives = in.receives_shadows != 0u;



    for (var i = 0; i < i32(directional_lights.count); i += 1) {
        let light = directional_lights.data[i];

        var shadow_value = 1.0;
        if (receives) {
            shadow_value = shadow_visibility(light.view_proj, light.shadow_layer, light.shadow_bias, light.shadow_scale, in.world_position, world_normal);
        }

        // Calculate light source vector
        // Negated after the multiply: naga's Metal backend leaves `-light.direction` a packed
        // vector (a vec3 followed by a scalar is packed), which can't be multiplied by a matrix.
        let l = -normalize(view_mat3 * light.direction);

        color += surface_shading(l, light.color, pixel, v, shadow_value);
    }

    // Hemisphere ambient light: the sky's light on surfaces facing up, the ground's on surfaces
    // facing down, blended in between.
    let up = normalize(view_mat3 * vec3<f32>(0.0, 1.0, 0.0));
    let facing_up = dot(pixel.normal, up) * 0.5 + 0.5;
    let ambient_light = mix(uniforms.ground.rgb, uniforms.sky.rgb, facing_up);
    // Ambient occlusion darkens only the ambient light: it stands for the surroundings blocking
    // light from the sky, not for the scene's lights, whose shadows are separate.
    let ambient = vec4<f32>(ambient_light * pixel.albedo.rgb * pixel.ambient_occlusion, pixel.albedo.a);

    // Point and spot lights, in view space like everything else here.
    for (var i = 0; i < i32(punctual_lights.count); i += 1) {
        let light = punctual_lights.data[i];

        let to_light = (camera.view * light.position).xyz - in.view_position.xyz;
        let distance = length(to_light);
        let l = to_light / max(distance, 0.0001);

        let axis = normalize((camera.view * light.direction).xyz + vec3<f32>(0.0, 0.0, 1e-9));
        let attenuation = distance_attenuation(distance, light.range)
            * cone_attenuation(dot(-l, axis), light.cone_scale, light.cone_offset);

        var shadow_value = 1.0;
        if (receives) {
            shadow_value = shadow_visibility(light.view_proj, light.shadow_layer, light.shadow_bias, light.shadow_scale, in.world_position, world_normal);
        }

        color += surface_shading(l, light.color * attenuation, pixel, v, shadow_value);
    }

    // Ambient light adds to the light from the scene's lights, and the surface's own light to
    // both, unshaded.
    let lit = ambient.rgb + color + material.emissive;
    return vec4<f32>(tonemap(lit * uniforms.exposure, uniforms.tonemapping), pixel.albedo.a);
    //return vec4<f32>((pixel.normal + 1.0) / 2.0, 1.0);
    //return vec4<f32>(pixel.metallic, 0.0, 0.0, 1.0);
    //return pixel.albedo;
}