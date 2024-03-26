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

struct ColorFilter {
    hue_rotation: f32,
    saturation_shift: f32,
    luminance_shift: f32,
    invert: u32,
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

@group(1) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(1)
var<uniform> color_filter: ColorFilter;

@group(2) @binding(0)
var<storage, read> gradient: Gradient;

@group(3) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(4) @binding(0)
var mask_texture: texture_2d<f32>;

@group(4) @binding(1)
var mask_sampler: sampler;

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let atlas_pixel = textureSample(atlas_texture, main_sampler, in.tex_coord);
    let main_pixel = textureSample(main_texture, main_sampler, in.tex_coord);

    let dim = textureDimensions(mask_texture);
    let mask_pixel = textureSample(mask_texture, mask_sampler, vec2<f32>(in.position.x / f32(dim.x), in.position.y / f32(dim.y)));

    let mode = in.mode & 31u;
    let masked = in.mode & (1u << 5u);

    var col = vec4<f32>(0.0);

    switch (mode) {
        case 0u: {
            col = vec4<f32>(in.color.r * atlas_pixel.a, in.color.g * atlas_pixel.a, in.color.b * atlas_pixel.a, atlas_pixel.a);
        }
        case 1u: {
            col = main_pixel;
        }
        case 2u: {
            col = in.color;
        }
        case 3u: {
            col = vec4<f32>(in.color.r * main_pixel.a, in.color.g * main_pixel.a, in.color.b * main_pixel.a, main_pixel.a);
        }
        case 4u: {
            col = vec4<f32>(atlas_pixel.r * atlas_pixel.a, atlas_pixel.g * atlas_pixel.a, atlas_pixel.b * atlas_pixel.a, atlas_pixel.a);
        }

        case 5u: {
            col = gradient_color(in.gradient_coord);
        }
        case 6u: {
            col = gradient_color(in.gradient_coord) * main_pixel.a;
        }
        case 7u: {
            col = gradient_color(in.gradient_coord) * atlas_pixel.a;
        }

        default: {
           col = vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    }

    var hsl = rgb_to_hsl(col.rgb / col.a);
    hsl.x = fract(hsl.x + color_filter.hue_rotation);
    hsl.y = clamp(hsl.y + color_filter.saturation_shift, 0.0, 1.0);
    hsl.z = clamp(hsl.z + color_filter.luminance_shift, 0.0, 1.0);

    // The color should be masked
    if (masked != 0u) {
        let a = mask_pixel.a * col.a;
        return vec4<f32>(hsl_to_rgb(hsl) * a, a);
    }
    return vec4<f32>(hsl_to_rgb(hsl) * col.a, col.a);
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

fn rgb_to_hsl(color: vec3<f32>) -> vec3<f32> {
    let c_max = max(max(color.r, color.g), color.b);
    let c_min = min(min(color.r, color.g), color.b);
    let delta = c_max - c_min;
    var hsl = vec3<f32>(0.0, 0.0, (c_max + c_min) / 2.0);

    if (delta != 0.0) {
        // Saturation
        if (hsl.z < 0.5) {
            hsl.y = delta / (c_max + c_min);
        } else {
            hsl.y = delta / (2.0 - c_max - c_min);
        }

        // Hue
        let delta_r = (((c_max - color.r) / 6.0) + (delta / 2.0)) / delta;
        let delta_g = (((c_max - color.g) / 6.0) + (delta / 2.0)) / delta;
        let delta_b = (((c_max - color.b) / 6.0) + (delta / 2.0)) / delta;

        if (color.r == c_max) {
            hsl.x = delta_b - delta_g;
        } else if (color.g == c_max) {
            hsl.x = (1.0 / 3.0) + delta_r - delta_b;
        } else {
            hsl.x = (2.0 / 3.0) + delta_g - delta_r;
        }

        // Ensure within [0.0, 1.0]
        // https://www.w3.org/TR/WGSL/#fract-builtin
        hsl.x = fract(hsl.x);
    }

    return hsl;
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
    if (hsl.y == 0.0) {
        return vec3<f32>(hsl.z);
    } else {
        var b = 0.0;
        if (hsl.z < 0.5) {
			b = hsl.z * (1.0 + hsl.y);
		} else {
			b = hsl.z + hsl.y - hsl.y * hsl.z;
		}

        let a = 2.0 * hsl.z - b;

        return vec3<f32>(
            hueRamp(a, b, hsl.x + (1.0 / 3.0)),
            hueRamp(a, b, hsl.x),
            hueRamp(a, b, hsl.x - (1.0 / 3.0)),
        );
    }
}

fn hueRamp(a: f32, b: f32, hue: f32) -> f32 {
   	let hue = fract(hue);

   	if ((6.0 * hue) < 1.0){
   		return a + (b - a) * 6.0 * hue;
   	} else if ((2.0 * hue) < 1.0) {
   		return b;
   	} else if ((3.0 * hue) < 2.0) {
   		return a + (b - a) * ((2.0 / 3.0) - hue) * 6.0;
   	} else {
   	    return a;
   	}
}