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
var<storage> point_lights: PointLights;
@group(1) @binding(2)
var primary_sampler: sampler;
@group(1) @binding(3)
var<storage> uniforms: Uniforms;

@vertex
fn main_vs(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) texture_coords_0: vec2<f32>,
    @location(4) texture_coords_1: vec2<f32>,
    @location(5) color_0: vec4<f32>,
    @location(6) color_1: vec4<f32>,
    @location(7) object_index: u32,
) -> VertexOutput {
    let object = objects[object_index];

    let model_view = camera.view * object.transform;
    let model_view_proj = camera.view_proj * object.transform;

    let mv_mat3 = mat3x3<f32>(model_view[0].xyz, model_view[1].xyz, model_view[2].xyz);

    let inv_scale_sq = mat3_inv_scale_squared(mv_mat3);

    var out: VertexOutput;
    out.position = model_view_proj * vec4<f32>(position, 1.0);
    out.normal = normalize(mv_mat3 * (inv_scale_sq * normal));
    out.tangent = normalize(mv_mat3 * (inv_scale_sq * tangent));
    out.texture_coords_0 = texture_coords_0;
    out.texture_coords_1 = texture_coords_1;
    out.color_0 = color_0;
    out.color_1 = color_1;
    out.material = object.material_index;
    out.view_position = model_view * vec4<f32>(position, 1.0);
    return out;
}

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let material = materials[in.material];
    var pixel: PixelData;

    let coords = (material.uv_transform0 * vec3<f32>(in.texture_coords_0, 1.0)).xy;
    let uvdx = dpdx(coords);
    let uvdy = dpdy(coords);

    pixel.albedo = get_albedo(material, primary_sampler, in);

    if (pixel.albedo.a < material.alpha_cutout) {
        discard;
    }
    pixel.normal = get_normal(in);

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



    for (var i = 0; i < i32(directional_lights.count); i += 1) {
        let light = directional_lights.data[i];

        // Get the shadow ndc coordinates, then convert to texture sample coordinates
        //let shadow_ndc = (light.view_proj * uniforms.inv_view * vs_out.view_position).xyz;
        //let shadow_flipped = (shadow_ndc.xy * 0.5) + 0.5;
        //let shadow_local_coords = vec2<f32>(shadow_flipped.x, 1.0 - shadow_flipped.y);

        // Texture sample coordinates of
        //var top_left = light.offset;
        //var top_right = top_left + light.size;
        //let shadow_coords = mix(top_left, top_right, shadow_local_coords);

        // The shadow is stored in an atlas, so we need to make sure we don't linear blend
        // across atlasses. We move our conditional borders in a half a pixel for standard
        // linear blending (so we're hitting texel centers on the edge). We move it an additional
        // pixel in so that our pcf5 offsets don't move off the edge of the atlasses.
        //let shadow_border = light.inv_resolution * 1.5;
        //top_left += shadow_border;
        //top_right -= shadow_border;

        var shadow_value = 1.0;
        //if (
        //    any(shadow_flipped >= top_left) && // XY lower
        //    any(shadow_flipped <= top_right) && // XY upper
        //    shadow_ndc.z >= 0.0 && // Z lower
        //    shadow_ndc.z <= 1.0 // Z upper
        //) {
        //    shadow_value = shadow_sample_pcf5(shadows, comparison_sampler, shadow_coords, shadow_ndc.z);
        //}

        // Calculate light source vector
        let l = normalize(view_mat3 * -light.direction);

        color += surface_shading(l, light.color, pixel, v, shadow_value * pixel.ambient_occlusion);
        //color = vec3<f32>(1.0, 0.0, 0.0);
    }

    let ambient = uniforms.ambient * pixel.albedo;
    let shaded = vec4<f32>(color, pixel.albedo.a);
    return max(ambient, shaded);
    //return vec4<f32>(pixel.metallic, 0.0, 0.0, 1.0);
    //return pixel.albedo;
}