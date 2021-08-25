struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Uniforms {
    transform: mat4x4<f32>;
};


[[group(1), binding(0)]]
var uniforms: Uniforms;

var window_size: vec2<f32> = vec2<f32>(1200.0, 900.0);

var filter: array<vec3<f32>,7> = array<vec3<f32>,7>(
    vec3<f32>(-3.0, -3.0, 0.1428571429),
    vec3<f32>(-2.0, -2.0, 0.1428571429),
    vec3<f32>(-1.0, -1.0, 0.1428571429),
    vec3<f32>(0.0, 0.0, 0.1428571429),
    vec3<f32>(1.0, 1.0, 0.1428571429),
    vec3<f32>(2.0, 2.0, 0.1428571429),
    vec3<f32>(3.0, 3.0, 0.1428571429),
);

[[group(0), binding(0)]]
var main_texture: texture_2d<f32>;

[[group(0), binding(1)]]
var main_sampler: sampler;

[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var color: vec4<f32> = vec4<f32>(0.0);
    for (var i: u32 = 0u; i < arrayLength(filter); i = i + 1u) {
        let texel_move = (vec2<f32>(1.0) / window_size) * filter[i].xy;
        color = color + filter[i].z * textureSample(main_texture, main_sampler, in.tex_coord + texel_move);
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