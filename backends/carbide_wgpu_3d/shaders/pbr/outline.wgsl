// Screen-space outlines around the silhouette of outlined objects, after the graphics crate's
// method (`references/3d/repos/graphics/src/shader_outline.wgsl`):
// 1. `outline_mask_*`: outlined meshes are drawn into a mask, each pixel holding its outline's
//    linear colour and its thickness in physical pixels; 0 where nothing is outlined.
// 2. `outline_horizontal`: for each pixel, the offset along its row to the nearest covered pixel.
// 3. `outline_composite`: from the row offsets of the rows around it, each uncovered pixel's
//    distance to the nearest covered pixel. Within the thickness it gets the outline, with a
//    one-pixel falloff that anti-aliases the outer edge. Blended onto the layer, on top of the
//    scene.
// The two passes make the cost per pixel linear in the thickness instead of quadratic.
//!include pbr/object.wgsl
//!include pbr/camera.wgsl

@group(0) @binding(1)
var<storage> objects: array<Object>;
@group(0) @binding(2)
var<storage> camera: Camera;

struct MaskOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) outline: vec4<f32>,
}

@vertex
fn outline_mask_vs(@location(0) position: vec3<f32>, @builtin(instance_index) object_index: u32) -> MaskOutput {
    let object = objects[object_index];
    var out: MaskOutput;
    out.position = camera.view_proj * object.transform * vec4<f32>(position, 1.0);
    out.outline = object.outline;
    return out;
}

@fragment
fn outline_mask_fs(in: MaskOutput) -> @location(0) vec4<f32> {
    return in.outline;
}

struct OutlineUniforms {
    // How far to search, in physical pixels: the thickest outline, plus one for the falloff.
    radius: i32,
}

// Written by the horizontal pass where no covered pixel is within the radius in the row.
const NONE_OFFSET: f32 = 10000.0;

@group(0) @binding(0) var mask: texture_2d<f32>;
@group(0) @binding(1) var<storage> outline_uniforms: OutlineUniforms;
@group(0) @binding(2) var horizontal: texture_2d<f32>;

// A triangle covering the whole target, with no vertex buffer.
@vertex
fn outline_fullscreen_vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(positions[index], 0.0, 1.0);
}

fn in_bounds(pixel: vec2<i32>, size: vec2<i32>) -> bool {
    return all(pixel >= vec2<i32>(0)) && all(pixel < size);
}

fn covered(pixel: vec2<i32>) -> bool {
    return textureLoad(mask, pixel, 0).a > 0.0;
}

@fragment
fn outline_horizontal(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(position.xy);
    let size = vec2<i32>(textureDimensions(mask));

    if (covered(pixel)) {
        return vec4<f32>(0.0);
    }

    // Outwards, so the first covered pixel found is the nearest.
    for (var d = 1; d <= outline_uniforms.radius; d += 1) {
        let left = pixel - vec2<i32>(d, 0);
        if (in_bounds(left, size) && covered(left)) {
            return vec4<f32>(f32(-d), 0.0, 0.0, 0.0);
        }

        let right = pixel + vec2<i32>(d, 0);
        if (in_bounds(right, size) && covered(right)) {
            return vec4<f32>(f32(d), 0.0, 0.0, 0.0);
        }
    }

    return vec4<f32>(NONE_OFFSET, 0.0, 0.0, 0.0);
}

@fragment
fn outline_composite(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(position.xy);
    let size = vec2<i32>(textureDimensions(mask));

    // The outline surrounds the silhouette; it doesn't cover it.
    if (covered(pixel)) {
        discard;
    }

    var color = vec3<f32>(0.0);
    var alpha = 0.0;

    for (var dy = -outline_uniforms.radius; dy <= outline_uniforms.radius; dy += 1) {
        let row = pixel + vec2<i32>(0, dy);
        if (!in_bounds(row, size)) {
            continue;
        }

        let dx = textureLoad(horizontal, row, 0).r;
        if (abs(dx) > f32(outline_uniforms.radius)) {
            continue;
        }

        let nearest = textureLoad(mask, row + vec2<i32>(i32(dx), 0), 0);
        let distance = length(vec2<f32>(dx, f32(dy)));

        // Fully covered out to the thickness, then fading over one pixel.
        let a = clamp(nearest.a + 1.0 - distance, 0.0, 1.0);
        if (a > alpha) {
            alpha = a;
            color = nearest.rgb;
        }
    }

    if (alpha <= 0.0) {
        discard;
    }

    // Premultiplied, as the layer is blended.
    return vec4<f32>(color * alpha, alpha);
}
