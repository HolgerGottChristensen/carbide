// Draws a cube's mip level from the level above it: each texel's direction meets the middle of
// four texels there, which a linear sample averages.
//!include environment/face.wgsl

struct Params {
    target_size: f32,
}

@group(0) @binding(0)
var source: texture_cube<f32>;
@group(0) @binding(1)
var source_sampler: sampler;
@group(0) @binding(2)
var<uniform> params: Params;

@fragment
fn downsample_fs(in: FaceOutput) -> @location(0) vec4<f32> {
    let d = face_direction(in.face, in.position.xy / params.target_size);
    return vec4<f32>(textureSampleLevel(source, source_sampler, d, 0.0).rgb, 1.0);
}
