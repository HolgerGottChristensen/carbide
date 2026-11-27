// Draws a cube face from an equirectangular panorama: the image's middle column is +X, its left
// and right edges -X, its top +Y; +Z is a quarter to the right of the middle, as three.js and
// Khronos's glTF Sample Viewer map panoramas.
//!include environment/face.wgsl

struct Params {
    // The face's size in texels.
    target_size: f32,
    // The panorama's mip level to read, so a large panorama isn't undersampled.
    source_level: f32,
}

@group(0) @binding(0)
var panorama: texture_2d<f32>;
@group(0) @binding(1)
var panorama_sampler: sampler;
@group(0) @binding(2)
var<uniform> params: Params;

@fragment
fn equirect_fs(in: FaceOutput) -> @location(0) vec4<f32> {
    let d = face_direction(in.face, in.position.xy / params.target_size);
    let u = 0.5 + atan2(d.z, d.x) / (2.0 * ENV_PI);
    let v = acos(clamp(d.y, -1.0, 1.0)) / ENV_PI;
    let color = textureSampleLevel(panorama, panorama_sampler, vec2<f32>(u, v), params.source_level).rgb;
    return vec4<f32>(color, 1.0);
}
