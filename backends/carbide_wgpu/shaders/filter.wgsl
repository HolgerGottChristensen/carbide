struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Uniforms {
    transform: mat4x4<f32>;
};

[[block]]
struct BlurUniforms {
    texture_size: vec2<f32>;
    number_of_blurs: u32;
    transform: [[stride(16)]] array<vec3<f32>>;
};

[[group(0), binding(0)]]
var main_texture: texture_2d<f32>;

[[group(0), binding(1)]]
var main_sampler: sampler;

[[group(0), binding(2)]]
var<storage> blur_uniforms: [[access(read)]] BlurUniforms; // This should change for wgpu 0.10


[[group(1), binding(0)]]
var uniforms: Uniforms;


[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var color: vec4<f32> = vec4<f32>(0.0);
    for (var i: u32 = 0u; i < blur_uniforms.number_of_blurs; i = i + 1u) {
        let texel_move = (vec2<f32>(1.0) / blur_uniforms.texture_size) * blur_uniforms.transform[i].xy;
        color = color + blur_uniforms.transform[i].z * textureSample(main_texture, main_sampler, in.tex_coord + texel_move);
    }
    return color;
}

[[stage(vertex)]]
fn main_vs(
    [[location(0)]] position: vec4<f32>,
    [[location(1)]] tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = tex_coord;
    out.position = uniforms.transform * position;
    return out;
}