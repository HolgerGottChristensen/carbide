struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2)]] mode: u32;
};

[[block]]
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
var uniforms: Uniforms;

var colors: array<vec4<f32>,3> = array<vec4<f32>,3>(
    vec4<f32>(1.0, 0.0, 0.0, 1.0),
    vec4<f32>(0.0, 1.0, 0.0, 1.0),
    vec4<f32>(0.0, 0.0, 1.0, 1.0),
);

var ratios: array<f32,3> = array<f32,3>(
    0.0,
    0.5,
    1.0,
);

[[stage(fragment)]]
fn main_fs(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let last = 2u; //gradient.num_colors - 1u;
    var t: f32;
    // Linear gradient
    let start = vec2<f32>(0.0, 0.0);
    let end = vec2<f32>(0.5, 0.0);
    t = dot(normalize(end - start), in.tex_coord - start) / length(end - start);

    // Radial gradient
    //let center = vec2<f32>(0.0, 0.0);
    //let distance = vec2<f32>(0.1, 0.0);
    //t = length(in.tex_coord - center) / length(center - distance);

    // Diamond gradient
    //let center = vec2<f32>(0.0, 0.0);
    //let end = vec2<f32>(0.5, 0.5);
    //let f = end - center;
    //let de = atan2(f.y, f.x);
    //let rot = mat2x2<f32>(vec2<f32>(cos(de), -sin(de)), vec2<f32>(sin(de), cos(de)));
    //let d = (rot*(in.tex_coord - center));
    //t = (abs(d.x) + abs(d.y)) / length(f);

    // Conic gradient
    //let center = vec2<f32>(-0.5, 0.0);
    //let end = vec2<f32>(0.0, 1.0);
    //let f = end - center;
    //let de = atan2(f.y, f.x) - 3.14159;
    //let rot = mat2x2<f32>(vec2<f32>(cos(de), -sin(de)), vec2<f32>(sin(de), cos(de)));
    //let d = rot*(in.tex_coord - center);
    //t = (atan2(d.y, d.x) * 180.0 / 3.14159 + 180.0) / 360.0;


    // Repeat
    //t = fract(t);

    // Mirror
    if( t < 0.0 ) {
        t = -t;
    }
    if((i32(t)&1) == 0) {
        t = fract(t);
    } else {
        t = 1.0 - fract(t);
    }

    // Clamp
    //t = clamp(t, 0.0, 1.0);



    t = clamp(t, ratios[0], ratios[last]);

    var j: u32;
    for( j = 1u; t > ratios[j]; j = j + 1u) {
        // Noop
    }

    let i = j - 1u;

    let a = (t - ratios[i]) / (ratios[j] - ratios[i]);

    var color: vec4<f32> = mix(colors[i], colors[j], a);

    return color;
}



[[stage(vertex)]]
fn main_vs(
    [[location(0)]] position: vec4<f32>,
    [[location(1)]] tex_coord: vec2<f32>,
    [[location(2)]] color: vec4<f32>,
    [[location(3)]] mode: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = (uniforms.transform * position).xy;
    out.position = uniforms.transform * position;
    out.color = color;
    out.mode = mode;
    return out;
}