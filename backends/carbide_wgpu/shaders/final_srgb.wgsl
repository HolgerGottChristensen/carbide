// A shader that takes an image and renders it to a target.
// The shader expects the target to be in srgb already, and does no conversions.

@group(0) @binding(0)
var main_texture: texture_2d<f32>;

@group(0) @binding(1)
var main_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    const size = 1.0;

    var positions = array<vec4<f32>, 6>(
        vec4<f32>(-size, -size, 0.0, 1.0),
        vec4<f32>( size, -size, 1.0, 1.0),
        vec4<f32>(-size,  size, 0.0, 0.0),

        vec4<f32>( size, -size, 1.0, 1.0),
        vec4<f32>( size,  size, 1.0, 0.0),
        vec4<f32>(-size,  size, 0.0, 0.0),
    );

    var position = positions[vertex_index];

    var out: VertexOutput;
    out.position = vec4<f32>(position.xy, 0.0, 1.0);
    out.tex_coord = position.zw;

    return out;
}

@fragment
fn fs_main(@location(0) position: vec2<f32>) -> @location(0) vec4<f32> {
    let texel = textureSample(main_texture, main_sampler, position);
    return vec4<f32>(texel.rgb, 1.0);
}
