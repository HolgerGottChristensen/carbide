// The shadow pass: draws the meshes that cast shadows into a light's shadow map, depth only.
//
// Cutout and blended materials are alpha-tested: a fragment whose albedo alpha is below the
// material's cutoff (cutout), or below 0.5 (blended), casts no shadow. Opaque materials don't
// sample anything.
//!include pbr/object.wgsl
//!include pbr/material.wgsl

@group(0) @binding(0)
var<storage> objects: array<Object>;
@group(0) @binding(1)
var<storage> light_view_proj: mat4x4<f32>;
@group(0) @binding(2)
var<storage> materials: array<Material>;
@group(0) @binding(3)
var primary_sampler: sampler;
@group(0) @binding(4)
var nearest_sampler: sampler;

// The material's textures, as the main pass binds them; only the albedo is read.
@group(1) @binding(0)
var albedo_tex: texture_2d<f32>;

struct ShadowOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords_0: vec2<f32>,
    @location(1) texture_coords_1: vec2<f32>,
    @location(2) color_0: vec4<f32>,
    @location(3) @interpolate(flat) material: u32,
}

@vertex
fn shadow_vs(
    @location(0) position: vec3<f32>,
    @location(3) texture_coords_0: vec2<f32>,
    @location(4) texture_coords_1: vec2<f32>,
    @location(5) color_0: vec4<f32>,
    @builtin(instance_index) object_index: u32,
) -> ShadowOutput {
    let object = objects[object_index];
    var out: ShadowOutput;
    out.position = light_view_proj * object.transform * vec4<f32>(position, 1.0);
    out.texture_coords_0 = texture_coords_0;
    out.texture_coords_1 = texture_coords_1;
    out.color_0 = color_0;
    out.material = object.material_index;
    return out;
}

@fragment
fn shadow_fs(in: ShadowOutput) {
    let material = materials[in.material];

    // The albedo map's coordinates: its set, transformed by its transform (slot 0), with their
    // derivatives taken before any branch.
    let second = (material.uv_sets & 1u) != 0u;
    let transform = material.map_transforms[0];
    let coords = (transform * vec3<f32>(select(in.texture_coords_0, in.texture_coords_1, second), 1.0)).xy;
    let ddx = dpdx(coords);
    let ddy = dpdy(coords);

    let blended = (material.flags & 0x8000u) != 0u;
    let cutout = material.alpha_cutout > 0.0;
    if (!blended && !cutout) {
        return;
    }

    // The albedo's alpha, as the main pass works it out (`albedo.wgsl`): the map's, times the
    // vertex colour's when the material uses them, times the material's.
    var alpha = material.albedo.a;
    if ((material.texture_enable & 0x1u) != 0u) {
        var texel: vec4<f32>;
        if ((material.flags & 0x4000u) != 0u) {
            texel = textureSampleGrad(albedo_tex, nearest_sampler, coords, ddx, ddy);
        } else {
            texel = textureSampleGrad(albedo_tex, primary_sampler, coords, ddx, ddy);
        }
        alpha *= texel.a;
    }
    if ((material.flags & 0x2u) != 0u) {
        alpha *= in.color_0.a;
    }

    let threshold = select(material.alpha_cutout, 0.5, blended);
    if (alpha < threshold) {
        discard;
    }
}
