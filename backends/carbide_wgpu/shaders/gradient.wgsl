struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Gradient {
    colors: array<vec4<f32>,16u>;
    ratios: array<f32,16u>;
    num_colors: u32;
    gradient_type: i32;
    repeat_mode: i32;
    start: vec2<f32>;
    end: vec2<f32>;
};

[[block]]
struct Uniforms {
    transform: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<storage, read> gradient: Gradient;

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
    let last = gradient.num_colors - 1u;
    var t: f32;
    // Linear gradient
    //let start = vec2<f32>(-1.0, 0.0);
    //let end = vec2<f32>(1.0, 0.0);
    //t = dot(normalize(gradient.end - gradient.start), in.tex_coord - gradient.start) / distance(gradient.start, gradient.end);

    if (distance(in.tex_coord, gradient.start) < 3.0) {
        discard;
    }
    if (distance(in.tex_coord, gradient.end) < 3.0) {
        discard;
    }


    // Radial gradient
    //let center = vec2<f32>(0.0, 0.0);
    //let distance = vec2<f32>(0.1, 0.0);
    //t = length(in.tex_coord - gradient.start) / distance(gradient.start, gradient.end);

    // Diamond gradient
    //let center = vec2<f32>(0.0, 0.0);
    //let end = vec2<f32>(0.5, 0.5);
    //let f = gradient.end - gradient.start;
    //let de = atan2(f.y, f.x);
    //let rot = mat2x2<f32>(vec2<f32>(cos(de), -sin(de)), vec2<f32>(sin(de), cos(de)));
    //let d = (rot*(in.tex_coord - gradient.start));
    //t = (abs(d.x) + abs(d.y)) / length(f);

    // Conic gradient
    //let center = vec2<f32>(-0.5, 0.0);
    //let end = vec2<f32>(0.0, 1.0);
    let f = gradient.end - gradient.start;
    let de = atan2(f.y, f.x) - 3.14159;
    let rot = mat2x2<f32>(vec2<f32>(cos(de), -sin(de)), vec2<f32>(sin(de), cos(de)));
    let d = rot*(in.tex_coord - gradient.start);
    t = (atan2(d.y, d.x) * 180.0 / 3.14159 + 180.0) / 360.0;


    // Repeat
    //t = fract(t);

    // Mirror
    //if( t < 0.0 ) {
    //    t = -t;
    //}
    //if((i32(t)&1) == 0) {
    //    t = fract(t);
    //} else {
    //    t = 1.0 - fract(t);
    //}

    // Clamp
    t = clamp(t, 0.0, 1.0);



    t = clamp(t, gradient.ratios[0], gradient.ratios[last]);

    var j: u32;
    for( j = 1u; t > gradient.ratios[j]; j = j + 1u) {
        // Noop
    }

    let i = j - 1u;

    let a = (t - gradient.ratios[i]) / (gradient.ratios[j] - gradient.ratios[i]);

    var color: vec4<f32> = mix(gradient.colors[i], gradient.colors[j], a);

    return color;
}



[[stage(vertex)]]
fn main_vs(
    [[location(0)]] position: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = position.xy;
    out.position = uniforms.transform * position;
    return out;
}