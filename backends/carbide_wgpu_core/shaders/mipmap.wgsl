// Draws one mip level from the level above it. A triangle covers the target, and each pixel
// samples the source at its centre with linear filtering, which averages the 2×2 texels under it.
// Through sRGB views, sampling decodes to linear and writing encodes back, so the average is taken
// in linear space.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    // One triangle, (0, 0), (2, 0), (0, 2) in UV, that covers the whole target.
    let x = f32((index << 1u) & 2u);
    let y = f32(index & 2u);

    var out: VertexOutput;
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(source, source_sampler, in.uv);
}
