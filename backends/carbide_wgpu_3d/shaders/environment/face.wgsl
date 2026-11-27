// Shared by the passes that build environment maps: a fullscreen triangle per cube face, the
// direction of a face's texel, and GGX importance sampling.
//
// Faces follow the cube-map convention WGSL samples with (+X, -X, +Y, -Y, +Z, -Z), with texel
// rows going down, as Bevy's `sample_cube_dir` (`bevy_pbr/src/render/utils.wesl`).

const ENV_PI = 3.14159265358979323846;

struct FaceOutput {
    @builtin(position) position: vec4<f32>,
    // The face being drawn: the draw's instance.
    @location(0) @interpolate(flat) face: u32,
}

// A triangle covering the target, for the face given as the instance.
@vertex
fn face_vs(@builtin(vertex_index) vertex: u32, @builtin(instance_index) face: u32) -> FaceOutput {
    let corner = vec2<f32>(f32((vertex << 1u) & 2u), f32(vertex & 2u));
    var out: FaceOutput;
    out.position = vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
    out.face = face;
    return out;
}

// The direction through `uv` (0 to 1, rows going down) on `face`, normalised.
fn face_direction(face: u32, uv: vec2<f32>) -> vec3<f32> {
    let c = 2.0 * uv - 1.0;
    var direction: vec3<f32>;
    switch face {
        case 0u: { direction = vec3<f32>(1.0, -c.y, -c.x); }
        case 1u: { direction = vec3<f32>(-1.0, -c.y, c.x); }
        case 2u: { direction = vec3<f32>(c.x, 1.0, c.y); }
        case 3u: { direction = vec3<f32>(c.x, -1.0, -c.y); }
        case 4u: { direction = vec3<f32>(c.x, -c.y, 1.0); }
        default: { direction = vec3<f32>(-c.x, -c.y, -1.0); }
    }
    return normalize(direction);
}

// The `i`th of `count` points of the Hammersley sequence, in [0, 1)².
fn hammersley(i: u32, count: u32) -> vec2<f32> {
    return vec2<f32>(f32(i) / f32(count), f32(reverseBits(i)) * 2.3283064365386963e-10);
}

// A half vector around +Z, distributed as GGX with `alpha` (roughness squared), from `xi`
// (Karis, "Real Shading in Unreal Engine 4").
fn importance_sample_ggx(xi: vec2<f32>, alpha: f32) -> vec3<f32> {
    let a2 = alpha * alpha;
    let phi = 2.0 * ENV_PI * xi.x;
    let cos_theta = sqrt((1.0 - xi.y) / (1.0 + (a2 - 1.0) * xi.y));
    let sin_theta = sqrt(max(1.0 - cos_theta * cos_theta, 0.0));
    return vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);
}

// The GGX normal distribution, with `alpha` the roughness squared.
fn d_ggx(noh: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let f = noh * noh * (a2 - 1.0) + 1.0;
    return a2 / (ENV_PI * f * f);
}

// A tangent frame around `n`: `n` is the third column.
fn frame(n: vec3<f32>) -> mat3x3<f32> {
    let up = select(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 0.0), abs(n.z) > 0.999);
    let t = normalize(cross(up, n));
    return mat3x3<f32>(t, cross(n, t), n);
}
