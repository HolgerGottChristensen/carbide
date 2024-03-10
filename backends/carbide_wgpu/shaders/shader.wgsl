struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) mode: u32,
    @location(3) gradient_coord: vec2<f32>,
}

struct Uniforms {
    transform: mat4x4<f32>,
}

struct Gradient {
    colors: array<vec4<f32>,16u>,
    ratios: array<f32,16u>,
    num_colors: u32,
    gradient_type: i32,
    repeat_mode: i32,
    start: vec2<f32>,
    end: vec2<f32>,
}

@group(0) @binding(0)
var main_texture: texture_2d<f32>;

@group(0) @binding(1)
var main_sampler: sampler;

@group(0) @binding(2)
var atlas_texture: texture_2d<f32>;


@group(1) @binding(0)
var<uniform> uniforms: Uniforms;

@group(2) @binding(0)
var<storage, read> gradient: Gradient;


@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let atlas_pixel = textureSample(atlas_texture, main_sampler, in.tex_coord);
    let main_pixel = textureSample(main_texture, main_sampler, in.tex_coord);

    switch (in.mode) {
        case 0u: {
            return vec4<f32>(in.color.r * atlas_pixel.a, in.color.g * atlas_pixel.a, in.color.b * atlas_pixel.a, atlas_pixel.a);
        }
        case 1u: {
            return main_pixel;
        }
        case 2u: {
            return in.color;
        }
        case 3u: {
            return vec4<f32>(in.color.r * main_pixel.a, in.color.g * main_pixel.a, in.color.b * main_pixel.a, main_pixel.a);
        }
        case 4u: {
            return vec4<f32>(atlas_pixel.r * atlas_pixel.a, atlas_pixel.g * atlas_pixel.a, atlas_pixel.b * atlas_pixel.a, atlas_pixel.a);
        }

        case 5u: {
            return gradient_color(in.gradient_coord);
        }
        case 6u: {
            return gradient_color(in.gradient_coord) * main_pixel.a;
        }
        case 7u: {
            return gradient_color(in.gradient_coord) * atlas_pixel.a;
        }

        default: {
           return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    }
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
    out.gradient_coord = position.xy;
    out.color = color;
    out.mode = mode;
    return out;
}

fn gradient_color(gradient_coord: vec2<f32>) -> vec4<f32> {
    let last = gradient.num_colors - 1u;
    var t: f32;

    switch (gradient.gradient_type) {
        case 0: {
            // Linear gradient
            t = dot(normalize(gradient.end - gradient.start), gradient_coord - gradient.start) / distance(gradient.start, gradient.end);
        }
        case 1: {
            // Radial gradient
            t = length(gradient_coord - gradient.start) / distance(gradient.start, gradient.end);
        }
        case 2: {
            // Diamond gradient
            let f = gradient.end - gradient.start;
            let de = atan2(f.y, f.x);
            let rot = mat2x2<f32>(vec2<f32>(cos(de), -sin(de)), vec2<f32>(sin(de), cos(de)));
            let d = (rot*(gradient_coord - gradient.start));
            t = (abs(d.x) + abs(d.y)) / length(f);
        }
        case 3: {
            // Conic gradient
            let f = gradient.end - gradient.start;
            let de = atan2(f.y, f.x) - 3.14159;
            let rot = mat2x2<f32>(vec2<f32>(cos(de), -sin(de)), vec2<f32>(sin(de), cos(de)));
            let d = rot*(gradient_coord - gradient.start);
            t = (atan2(d.y, d.x) * 180.0 / 3.14159 + 180.0) / 360.0;
        }
        default: {
            t = 1.0;
        }
    }

    switch (gradient.repeat_mode) {
        case 0: {
            // Clamp
            t = clamp(t, 0.0, 1.0);
        }
        case 1: {
            // Repeat
            t = fract(t);
        }
        case 2: {
            // Mirror
            if( t < 0.0 ) {
                t = -t;
            }
            if((i32(t)&1) == 0) {
                t = fract(t);
            } else {
                t = 1.0 - fract(t);
            }
        }
        default: {
            // Default to clamp
            t = clamp(t, 0.0, 1.0);
        }
    }

    t = clamp(t, gradient.ratios[0], gradient.ratios[last]);

    var j: u32;
    for( j = 1u; t > gradient.ratios[j]; j = j + 1u) {
        // Noop - but increate J while the ratios is less than t
    }

    let i = j - 1u;

    let a = (t - gradient.ratios[i]) / (gradient.ratios[j] - gradient.ratios[i]);

    var color: vec4<f32> = mix(gradient.colors[i], gradient.colors[j], a);

    color.r = color.r * color.a;
    color.g = color.g * color.a;
    color.b = color.b * color.a;

    return color;
}