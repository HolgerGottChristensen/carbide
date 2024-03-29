struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) mode: u32,
}

struct Uniforms {
    transform: mat4x4<f32>,
}

struct SizeUniforms {
    size: vec2<f32>,
}

struct FilterUniforms {
    texture_size: vec2<f32>,
    length: u32,
    transform: array<vec3<f32>>,
}

@group(0) @binding(0)
var main_texture: texture_2d<f32>;

@group(0) @binding(1)
var main_sampler: sampler;

@group(1) @binding(0)
var<storage, read> filter_uniforms: FilterUniforms;

@group(2) @binding(0)
var<uniform> uniforms: Uniforms;

@group(3) @binding(0)
var mask_texture: texture_2d<f32>;

@group(3) @binding(1)
var mask_sampler: sampler;

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = vec4<f32>(0.0);
    let texel_size: vec2<f32> = vec2<f32>(1.0) / filter_uniforms.texture_size;

    let dim = textureDimensions(mask_texture);
    let mask_pixel = textureSample(mask_texture, mask_sampler, vec2<f32>(in.position.x / f32(dim.x), in.position.y / f32(dim.y)));

    for (var i: u32 = 0u; i < filter_uniforms.length; i = i + 1u) {
        let texel_move = texel_size * filter_uniforms.transform[i].xy;
        color = color + filter_uniforms.transform[i].z * textureSample(main_texture, main_sampler, in.tex_coord + texel_move);
    }

    let mode = in.mode & 31u;
    let masked = in.mode & (1u << 5u);

    if (masked != 0u && mask_pixel.a == 0.0) {
        discard;
    }

    if (mode == 1u) {
        let a = clamp(color.a, 0.0, 1.0);
        let s = vec4<f32>(in.color.rgb * a, a);
        return s;
    }

    if (mode == 2u) {
        let a = 1.0 - clamp(color.a, 0.0, 1.0);
        let s = vec4<f32>(in.color.rgb * a, a);
        return s;
    }

    return color;
}

@vertex
fn main_vs(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) mode: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = tex_coord;
    out.position = uniforms.transform * position;
    out.color = color;
    out.mode = mode;
    return out;
}