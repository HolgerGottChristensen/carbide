struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2)]] mode: u32;
};

struct Uniforms {
    transform: mat4x4<f32>;
};

[[group(0), binding(0)]]
var main_texture: texture_2d<f32>;

[[group(0), binding(1)]]
var main_sampler: sampler;

[[group(0), binding(2)]]
var atlas_texture: texture_2d<f32>;


[[group(1), binding(0)]]
var<uniform> uniforms: Uniforms;


[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let atlas_pixel = textureSample(atlas_texture, main_sampler, in.tex_coord);
    let main_pixel = textureSample(main_texture, main_sampler, in.tex_coord);

    if (in.mode == 0u) {
        return vec4<f32>(in.color.r * atlas_pixel.a, in.color.g * atlas_pixel.a, in.color.b * atlas_pixel.a, atlas_pixel.a);
    }
    if (in.mode == 1u) {
        return main_pixel;
    }
    if (in.mode == 2u) {
        return in.color;
    }
    if (in.mode == 3u) {
        return vec4<f32>(in.color.r * main_pixel.a, in.color.g * main_pixel.a, in.color.b * main_pixel.a, main_pixel.a);
    }

    return vec4<f32>(atlas_pixel.r * atlas_pixel.a, atlas_pixel.g * atlas_pixel.a, atlas_pixel.b * atlas_pixel.a, atlas_pixel.a);
}

[[stage(vertex)]]
fn main_vs(
    [[location(0)]] position: vec4<f32>,
    [[location(1)]] tex_coord: vec2<f32>,
    [[location(2)]] color: vec4<f32>,
    [[location(3)]] mode: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = tex_coord;
    out.position = uniforms.transform * position;
    out.color = color;
    out.mode = mode;
    return out;
}