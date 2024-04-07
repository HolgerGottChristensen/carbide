struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) mode: u32,
    @location(3) gradient_coord: vec2<f32>,
    @location(4) line_coords: vec4<f32>,
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
    mode: u32,
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
        case 8u: {
            let dir = in.line_coords.zw - in.line_coords.xy;
            let len = length(in.line_coords.zw - in.line_coords.xy);

            let s = dot(in.position.xy, normalize(dir));

            let s0 = dot(in.gradient_coord.xy - in.line_coords.xy, normalize(dir));

            if (s0 > len) {
                return vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }


            if (s0 < 0.0) {
                return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }

            if (fract(s / 40.0) >= 0.5) {
                return vec4<f32>(0.0, 0.0, 1.0, 1.0);
            }

            col = in.color;
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);
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
    @location(4) line_coords: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = tex_coord;
    out.position = uniforms.transform * position;
    out.gradient_coord = position.xy;
    out.color = color;
    out.mode = mode;
    let p0 = vec4<f32>(line_coords.xy, 0.0, 1.0);
    let p1 = vec4<f32>(line_coords.zw, 0.0, 1.0);
    out.line_coords = vec4<f32>(p0.xy, p1.xy);
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

    var col = vec4<f32>(0.0);

    switch (gradient.mode) {
        // OKLAB
        case 1u: {
            let col1 = linear_srgb_to_oklab(gradient.colors[i].rgb);
            let col2 = linear_srgb_to_oklab(gradient.colors[j].rgb);

            col = vec4<f32>(oklab_to_linear_srgb(mix(col1, col2, a)), mix(gradient.colors[i].a, gradient.colors[j].a, a));
        }
        // Srgb
        case 2u: {
            let col1 = linear_to_srgb(gradient.colors[i]);
            let col2 = linear_to_srgb(gradient.colors[j]);

            col = srgb_to_linear(mix(col1, col2, a));
        }
        // Xyz
        case 3u: {
            let col1 = rgb_to_xyz(gradient.colors[i].rgb);
            let col2 = rgb_to_xyz(gradient.colors[j].rgb);

            col = vec4<f32>(xyz_to_rgb(mix(col1, col2, a)), mix(gradient.colors[i].a, gradient.colors[j].a, a));
        }
        // cielab
        case 4u: {
            let col1 = rgb_to_cielab(gradient.colors[i].rgb);
            let col2 = rgb_to_cielab(gradient.colors[j].rgb);

            col = vec4<f32>(cielab_to_rgb(mix(col1, col2, a)), mix(gradient.colors[i].a, gradient.colors[j].a, a));
        }
        // HSL
        case 5u: {
            var col1 = rgb_to_hsl(gradient.colors[i].rgb);
            var col2 = rgb_to_hsl(gradient.colors[j].rgb);

            if (col2.x - col1.x > 0.5) {
                col1.x = col1.x + 1.0;
                col = vec4<f32>(hsl_to_rgb(mix(col1, col2, a)), mix(gradient.colors[i].a, gradient.colors[j].a, a));
            } else if (col1.x - col2.x  > 0.5) {
                col2.x = col2.x + 1.0;
                col = vec4<f32>(hsl_to_rgb(mix(col1, col2, a)), mix(gradient.colors[i].a, gradient.colors[j].a, a));
            } else {
                col = vec4<f32>(hsl_to_rgb(mix(col1, col2, a)), mix(gradient.colors[i].a, gradient.colors[j].a, a));
            }
        }
        // Linear space
        default: {
            col = vec4<f32>(mix(gradient.colors[i].rgb, gradient.colors[j].rgb, a), mix(gradient.colors[i].a, gradient.colors[j].a, a));
        }
    }

    col.r = col.r * col.a;
    col.g = col.g * col.a;
    col.b = col.b * col.a;

    return col;
}

fn linear_srgb_to_oklab(c: vec3<f32>) -> vec3<f32> {
    let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;
    let l_ = pow(l, 1.0/3.0);
    let m_ = pow(m, 1.0/3.0);
    let s_ = pow(s, 1.0/3.0);

    return vec3<f32>(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    );
}

fn oklab_to_linear_srgb(c: vec3<f32>) -> vec3<f32> {
    let l_ = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
    let m_ = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
    let s_ = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    return vec3<f32>(
         4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    );
}

/// Converts a color from linear to sRGB color space.
fn linear_to_srgb(linear_: vec4<f32>) -> vec4<f32> {
    var rgb: vec3<f32> = linear_.rgb;
    if( linear_.a > 0.0 ) {
        rgb = rgb / linear_.a;
    }
    let a = 12.92 * rgb;
    let b = 1.055 * pow(rgb, vec3<f32>(1.0 / 2.4)) - 0.055;
    let c = step(vec3<f32>(0.0031308), rgb);
    return vec4<f32>(mix(a, b, c) * linear_.a, linear_.a);
}

/// Converts a color from sRGB to linear color space.
fn srgb_to_linear(srgb: vec4<f32>) -> vec4<f32> {
    var rgb: vec3<f32> = srgb.rgb;
    if( srgb.a > 0.0 ) {
        rgb = rgb / srgb.a;
    }
    let a = rgb / 12.92;
    let b = pow((rgb + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    let c = step(vec3<f32>(0.04045), rgb);
    return vec4<f32>(mix(a, b, c) * srgb.a, srgb.a);
}

const D65_WHITE: vec3<f32> = vec3<f32>(0.95045592705, 1.0, 1.08905775076);

fn xyz_to_lab_f(x: f32) -> f32 {
    //      (24/116)^3
    if (x > 0.00885645167) {
        return pow(x, 0.333333333);
    } else {
        //     1/(3*(6/29)^2)      4/29
        return 7.78703703704 * x + 0.13793103448;
    }
}

fn xyz_to_lab(c: vec3<f32>) -> vec3<f32> {
    var xyz_scaled = c / D65_WHITE;
    xyz_scaled = vec3<f32>(
        xyz_to_lab_f(xyz_scaled.x),
        xyz_to_lab_f(xyz_scaled.y),
        xyz_to_lab_f(xyz_scaled.z)
    );

    return vec3<f32>(
        (116.0 * xyz_scaled.y) - 16.0,
        500.0 * (xyz_scaled.x - xyz_scaled.y),
        200.0 * (xyz_scaled.y - xyz_scaled.z)
    );
}


fn lab_to_xyz_f(x: f32) -> f32 {
    if (x > 0.206897) {
        return x * x * x;
    } else {
        //      3*(6/29)^2           4/29
        return (0.12841854934 * (x - 0.137931034));
    }
}

fn lab_to_xyz(c: vec3<f32>) -> vec3<f32> {
    let w = (c.x + 16.0) / 116.0;
    return D65_WHITE * vec3<f32>(
        lab_to_xyz_f(w + c.y / 500.0),
        lab_to_xyz_f(w),
        lab_to_xyz_f(w - c.z / 200.0)
    );
}

fn cielab_to_rgb(c: vec3<f32>) -> vec3<f32> {
    return xyz_to_rgb(lab_to_xyz(c));
}

fn rgb_to_cielab(c: vec3<f32>) -> vec3<f32> {
    return xyz_to_lab(rgb_to_xyz(c));
}

// RGB<->XYZ
// from IEC 61966-2-1:1999/AMD1:2003 (sRGB color amendment 1)
const RGB_TO_XYZ_M: mat3x3<f32> = mat3x3<f32>(
    vec3<f32>(0.4124, 0.3576, 0.1805),
    vec3<f32>(0.2126, 0.7152, 0.0722),
    vec3<f32>(0.0193, 0.1192, 0.9505),
);
const XYZ_TO_RGB_M: mat3x3<f32> = mat3x3<f32>(
    vec3<f32>(3.2406255, -1.5372080, -0.4986286),
    vec3<f32>(-0.9689307, 1.8757561, 0.0415175),
    vec3<f32>(0.0557101, -0.2040211, 1.0569959),
);

/// A CIE 1931 XYZ color.
fn rgb_to_xyz(c: vec3<f32>) -> vec3<f32> {
    return c * RGB_TO_XYZ_M;
}

fn xyz_to_rgb(c: vec3<f32>) -> vec3<f32> {
    return c * XYZ_TO_RGB_M;
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